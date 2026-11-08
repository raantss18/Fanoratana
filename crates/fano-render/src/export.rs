//! Export : SVG (la scène telle quelle), PNG (resvg) et PDF (svg2pdf, une page par page de leçon).
//! Les polices voyagent dans le PDF ; le texte y reste du texte, cherchable et copiable.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;

use base64::Engine as _;
use fano_core::{Couleur, Lecon, Point};

use crate::polices::{CAVEAT, SYMBOLES};
use crate::scene::{self, Prim};

fn couleur(c: Couleur) -> (String, f32) {
    (c.hex(), c.alpha())
}

fn chemin(pts: &[Point], ferme: bool) -> String {
    let mut d = String::with_capacity(pts.len() * 14);
    for (i, p) in pts.iter().enumerate() {
        let _ = write!(d, "{}{:.2} {:.2}", if i == 0 { "M" } else { "L" }, p.x, p.y);
    }
    if ferme {
        d.push('Z');
    }
    d
}

fn echapper(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn mime(fichier: &str) -> &'static str {
    match fichier.rsplit('.').next().map(str::to_lowercase).as_deref() {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "application/octet-stream",
    }
}

/// La scène d'une page en SVG autonome (images incluses en `data:`).
pub fn svg(prims: &[Prim], w: f32, h: f32, coffre: &Path) -> String {
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\n"
    );
    for p in prims {
        match p {
            Prim::Remplir { pts, couleur: c } => {
                let (hex, a) = couleur(*c);
                let _ = writeln!(
                    s,
                    "<path d=\"{}\" fill=\"{hex}\" fill-opacity=\"{a:.3}\"/>",
                    chemin(pts, true)
                );
            }
            Prim::Trace {
                pts,
                epaisseur,
                couleur: c,
            } => {
                let (hex, a) = couleur(*c);
                let _ = writeln!(
                    s,
                    "<path d=\"{}\" fill=\"none\" stroke=\"{hex}\" stroke-opacity=\"{a:.3}\" stroke-width=\"{epaisseur:.2}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
                    chemin(pts, false)
                );
            }
            Prim::Texte {
                pos,
                texte,
                taille,
                couleur: c,
                police,
            } => {
                let (hex, a) = couleur(*c);
                let _ = writeln!(
                    s,
                    "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"{}\" font-size=\"{taille:.2}\" fill=\"{hex}\" fill-opacity=\"{a:.3}\" xml:space=\"preserve\">{}</text>",
                    pos.x,
                    pos.y,
                    police.famille(),
                    echapper(texte)
                );
            }
            Prim::Image { fichier, rect } => {
                if let Ok(octets) = std::fs::read(coffre.join(fichier)) {
                    let b64 = base64::engine::general_purpose::STANDARD.encode(octets);
                    let _ = writeln!(
                        s,
                        "<image x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" preserveAspectRatio=\"xMidYMid meet\" xlink:href=\"data:{};base64,{b64}\"/>",
                        rect.min.x,
                        rect.min.y,
                        rect.width(),
                        rect.height(),
                        mime(fichier)
                    );
                }
            }
        }
    }
    s.push_str("</svg>\n");
    s
}

fn options() -> usvg::Options<'static> {
    let mut o = usvg::Options::default();
    let db = o.fontdb_mut();
    db.load_font_data(CAVEAT.to_vec());
    db.load_font_data(SYMBOLES.to_vec());
    db.set_cursive_family(crate::polices::FAMILLE_MANUSCRIT);
    o
}

fn arbre(svg: &str) -> Result<usvg::Tree, String> {
    usvg::Tree::from_str(svg, &options()).map_err(|e| e.to_string())
}

/// Le SVG d'une page de leçon.
pub fn svg_page(lecon: &Lecon, i: usize, coffre: &Path) -> Option<String> {
    let page = lecon.pages.get(i)?;
    let (w, h) = lecon.format.taille();
    Some(svg(&scene::page(page, lecon.format), w, h, coffre))
}

/// Une page en PNG, à `echelle` (2 = 144 dpi).
pub fn png_page(lecon: &Lecon, i: usize, coffre: &Path, echelle: f32) -> Result<Vec<u8>, String> {
    let s = svg_page(lecon, i, coffre).ok_or("page absente")?;
    let t = arbre(&s)?;
    let taille = t.size().to_int_size().scale_by(echelle).ok_or("taille")?;
    let mut pix = tiny_skia::Pixmap::new(taille.width(), taille.height()).ok_or("image trop grande")?;
    resvg::render(
        &t,
        tiny_skia::Transform::from_scale(echelle, echelle),
        &mut pix.as_mut(),
    );
    pix.encode_png().map_err(|e| e.to_string())
}

/// Toute la leçon en PDF, une page PDF par page.
pub fn pdf_lecon(lecon: &Lecon, coffre: &Path) -> Result<Vec<u8>, String> {
    use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str, TextStr};
    let (w, h) = lecon.format.taille();
    let mut alloc = Ref::new(1);
    let catalogue = alloc.bump();
    let arbre_pages = alloc.bump();
    let info = alloc.bump();
    let mut pdf = Pdf::new();
    let mut pages = Vec::new();
    let mut morceaux = Vec::new();
    for i in 0..lecon.pages.len() {
        let s = svg_page(lecon, i, coffre).ok_or("page absente")?;
        let t = arbre(&s)?;
        let (chunk, id) =
            svg2pdf::to_chunk(&t, svg2pdf::ConversionOptions::default()).map_err(|e| e.to_string())?;
        let mut table = HashMap::new();
        let chunk = chunk.renumber(|old| *table.entry(old).or_insert_with(|| alloc.bump()));
        let id = table[&id];
        let page_id = alloc.bump();
        let contenu_id = alloc.bump();
        let mut c = Content::new();
        c.transform([w, 0.0, 0.0, h, 0.0, 0.0]).x_object(Name(b"S"));
        pages.push((page_id, contenu_id, id, c.finish()));
        morceaux.push(chunk);
    }
    pdf.catalog(catalogue).pages(arbre_pages);
    pdf.pages(arbre_pages)
        .kids(pages.iter().map(|p| p.0))
        .count(pages.len() as i32);
    for (page_id, contenu_id, svg_id, contenu) in &pages {
        let mut p = pdf.page(*page_id);
        p.media_box(Rect::new(0.0, 0.0, w, h));
        p.parent(arbre_pages);
        p.contents(*contenu_id);
        p.resources().x_objects().pair(Name(b"S"), *svg_id);
        p.finish();
        pdf.stream(*contenu_id, contenu);
    }
    pdf.document_info(info)
        .title(TextStr(&lecon.titre))
        .producer(TextStr("Fanoratana"));
    let _ = Str(b"");
    for m in &morceaux {
        pdf.extend(m);
    }
    Ok(pdf.finish())
}

/// Toutes les pages en PNG, dans un dossier : `<base>-1.png`, `<base>-2.png`…
pub fn png_lecon(
    lecon: &Lecon,
    coffre: &Path,
    dossier: &Path,
    base: &str,
) -> Result<Vec<std::path::PathBuf>, String> {
    let mut faits = Vec::new();
    for i in 0..lecon.pages.len() {
        let octets = png_page(lecon, i, coffre, 2.0)?;
        let p = if lecon.pages.len() == 1 {
            dossier.join(format!("{base}.png"))
        } else {
            dossier.join(format!("{base}-{}.png", i + 1))
        };
        std::fs::write(&p, octets).map_err(|e| e.to_string())?;
        faits.push(p);
    }
    Ok(faits)
}

/// Pour l'interface : les polices partagées, prêtes pour usvg.
pub fn base_polices() -> Arc<usvg::fontdb::Database> {
    options().fontdb
}

#[cfg(test)]
mod tests {
    use super::*;
    use fano_core::{Element, Format, Forme, FormeKind, Texte};

    fn lecon() -> Lecon {
        let mut l = Lecon::nouvelle("Essai", Format::A4);
        l.pages[0].elements.push(Element::Texte(Texte {
            pos: Point::new(72.0, 60.0),
            contenu: "Le théorème de Pythagore : $a^2+b^2=c^2$".into(),
            taille: 22.0,
            couleur: Couleur::BLEU,
            largeur: 440.0,
            titre: 1,
            encadre: fano_core::Encadre::Definition,
            source: vec![],
            alternatives: vec![],
        }));
        l.pages[0].elements.push(Element::Forme(Forme {
            kind: FormeKind::Triangle,
            points: vec![
                Point::new(100.0, 300.0),
                Point::new(300.0, 300.0),
                Point::new(100.0, 150.0),
            ],
            couleur: Couleur::ENCRE,
            epaisseur: 2.0,
            rempli: true,
            graine: 3,
            etiquette: String::new(),
            source: vec![],
        }));
        l.pages.push(fano_core::Page::default());
        l
    }

    #[test]
    fn png_et_pdf() {
        let dir = tempfile::tempdir().unwrap();
        let l = lecon();
        let png = png_page(&l, 0, dir.path(), 1.0).unwrap();
        assert_eq!(&png[1..4], b"PNG");
        let pdf = pdf_lecon(&l, dir.path()).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        let texte = String::from_utf8_lossy(&pdf);
        assert!(texte.contains("/Count 2"), "deux pages");
        if let Ok(d) = std::env::var("FANO_EXPORT_ESSAI") {
            std::fs::write(
                format!("{d}/essai.png"),
                png_page(&l, 0, dir.path(), 2.0).unwrap(),
            )
            .unwrap();
            std::fs::write(format!("{d}/essai.pdf"), pdf).unwrap();
        }
    }
}
