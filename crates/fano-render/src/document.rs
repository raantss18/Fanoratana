//! Un document à annoter (ADR 0015) : un PDF devient une image par page, une photo devient une page.
//!
//! Les pages du PDF sont rendues par hayro (Rust pur, MIT OR Apache-2.0) à 2× (144 dpi) : nettes sur la
//! tablette, légères dans le coffre. Toutes les pages d'une leçon ont la taille de la première ; une page
//! d'une autre taille est centrée sur du blanc, à la même proportion que les autres.

use std::sync::Arc;

/// Une page prête : image PNG à la proportion de la leçon.
pub struct PageDocument {
    pub png: Vec<u8>,
}

/// Un document prêt à devenir une leçon : taille des pages en points, et les pages.
pub struct Document {
    pub largeur: f32,
    pub hauteur: f32,
    pub pages: Vec<PageDocument>,
}

/// Au-delà, on refuse plutôt que de remplir la tablette (un livre entier se découpe avant).
pub const PAGES_MAX: usize = 200;
/// Points → pixels des pages rendues (2 = 144 dpi).
const ECHELLE: f32 = 2.0;

/// Est-ce un PDF ? (signature `%PDF-` au début)
pub fn est_pdf(octets: &[u8]) -> bool {
    octets.len() >= 5 && octets[..1024.min(octets.len())].windows(5).any(|w| w == b"%PDF-")
}

/// Rend toutes les pages d'un PDF ; `progres(fait, total)` après chaque page.
pub fn pdf(octets: &[u8], progres: impl Fn(usize, usize)) -> Result<Document, String> {
    let doc = hayro::hayro_syntax::Pdf::new(Arc::new(octets.to_vec()))
        .map_err(|e| format!("PDF illisible ({e:?})"))?;
    let pages = doc.pages();
    let n = pages.len();
    if n == 0 {
        return Err("PDF sans page".into());
    }
    if n > PAGES_MAX {
        return Err(format!("PDF de {n} pages : au plus {PAGES_MAX}"));
    }
    let (largeur, hauteur) = pages[0].render_dimensions();
    let (largeur, hauteur) = (largeur.max(1.0), hauteur.max(1.0));
    let cache = hayro::RenderCache::new();
    let reglages = hayro::hayro_interpret::InterpreterSettings::default();
    let mut res = Vec::with_capacity(n);
    for (i, p) in pages.iter().enumerate() {
        let (w, h) = p.render_dimensions();
        // La page tient dans le cadre de la leçon, sans être déformée.
        let k = (largeur / w.max(1.0)).min(hauteur / h.max(1.0)) * ECHELLE;
        let rs = hayro::RenderSettings {
            x_scale: k,
            y_scale: k,
            ..Default::default()
        };
        let pm = hayro::render(p, &cache, &reglages, &rs);
        let png = sur_blanc(
            pm.data_as_u8_slice(),
            pm.width() as u32,
            pm.height() as u32,
            (largeur * ECHELLE).round() as u32,
            (hauteur * ECHELLE).round() as u32,
        )?;
        res.push(PageDocument { png });
        progres(i + 1, n);
    }
    Ok(Document {
        largeur,
        hauteur,
        pages: res,
    })
}

/// Une image (photo d'un exercice, scan) : une page, large de 595 pt (A4) si elle est en hauteur,
/// 842 pt si elle est en largeur.
pub fn image(largeur_px: f32, hauteur_px: f32) -> (f32, f32) {
    let (w, h) = (largeur_px.max(1.0), hauteur_px.max(1.0));
    let l = if w > h { 842.0 } else { 595.0 };
    (l, l * h / w)
}

/// Pose les pixels RGBA prémultipliés (`w` × `h`) au centre d'un fond blanc `lw` × `lh`, en PNG.
fn sur_blanc(rgba: &[u8], w: u32, h: u32, lw: u32, lh: u32) -> Result<Vec<u8>, String> {
    let (lw, lh) = (lw.max(w).max(1), lh.max(h).max(1));
    let mut fond = tiny_skia::Pixmap::new(lw, lh).ok_or("page trop grande")?;
    fond.fill(tiny_skia::Color::WHITE);
    if w > 0 && h > 0 {
        let page = tiny_skia::Pixmap::from_vec(
            rgba.to_vec(),
            tiny_skia::IntSize::from_wh(w, h).ok_or("taille de page")?,
        )
        .ok_or("pixels de page")?;
        fond.draw_pixmap(
            ((lw - w) / 2) as i32,
            ((lh - h) / 2) as i32,
            page.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            tiny_skia::Transform::identity(),
            None,
        );
    }
    fond.encode_png().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un PDF de deux pages A4, la seconde en paysage, écrit par pdf-writer.
    fn deux_pages() -> Vec<u8> {
        use pdf_writer::{Content, Finish, Pdf, Rect, Ref};
        let mut pdf = Pdf::new();
        let (cat, arbre) = (Ref::new(1), Ref::new(2));
        pdf.catalog(cat).pages(arbre);
        pdf.pages(arbre).kids([Ref::new(3), Ref::new(5)]).count(2);
        for (page, contenu, (w, h)) in [(3, 4, (595.0, 842.0)), (5, 6, (842.0, 595.0))] {
            let mut p = pdf.page(Ref::new(page));
            p.media_box(Rect::new(0.0, 0.0, w, h))
                .parent(arbre)
                .contents(Ref::new(contenu));
            p.finish();
            let mut c = Content::new();
            // Un carré noir en haut à gauche.
            c.set_fill_rgb(0.0, 0.0, 0.0)
                .rect(50.0, h - 150.0, 100.0, 100.0)
                .fill_nonzero();
            pdf.stream(Ref::new(contenu), &c.finish());
        }
        pdf.finish()
    }

    #[test]
    fn un_pdf_devient_une_image_par_page() {
        let octets = deux_pages();
        assert!(est_pdf(&octets));
        let d = pdf(&octets, |_, _| {}).unwrap();
        assert_eq!((d.largeur.round(), d.hauteur.round()), (595.0, 842.0));
        assert_eq!(d.pages.len(), 2);
        for p in &d.pages {
            let img = tiny_skia::Pixmap::decode_png(&p.png).unwrap();
            // Toutes les pages à la taille de la première (2 × 595 × 842).
            assert_eq!((img.width(), img.height()), (1190, 1684));
        }
        // Page 1 : le carré noir est là, le reste est blanc.
        let p1 = tiny_skia::Pixmap::decode_png(&d.pages[0].png).unwrap();
        let px = |x: u32, y: u32| p1.pixel(x, y).unwrap();
        assert!(px(200, 200).red() < 30, "carré noir");
        assert!(px(1000, 1500).red() > 220, "fond blanc");
    }

    #[test]
    fn ce_qui_n_est_pas_un_pdf_est_refuse() {
        assert!(!est_pdf(b"\x89PNG\r\n"));
        assert!(pdf(b"%PDF-1.7 n'importe quoi", |_, _| {}).is_err());
    }

    #[test]
    fn une_photo_garde_sa_proportion() {
        assert_eq!(image(3000.0, 4000.0), (595.0, 595.0 * 4.0 / 3.0));
        assert_eq!(image(4000.0, 3000.0), (842.0, 842.0 * 0.75));
    }

    /// Du PDF reçu au PDF annoté : le fond et l'écriture sont tous les deux dans l'export.
    #[test]
    fn pdf_annote_puis_exporte() {
        use fano_core::{Couleur, Element, Lecon, PointEncre, Trait};
        let d = pdf(&deux_pages(), |_, _| {}).unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("medias")).unwrap();
        let mut images = vec![];
        for (i, p) in d.pages.iter().enumerate() {
            let f = format!("medias/p{i}.png");
            std::fs::write(dir.path().join(&f), &p.png).unwrap();
            images.push(f);
        }
        let mut l = Lecon::annotee("Sujet", d.largeur, d.hauteur, images);
        // Un trait rouge épais en bas de la page 1.
        let t = Trait {
            points: (0..50)
                .map(|x| PointEncre {
                    x: 100.0 + x as f32 * 8.0,
                    y: 700.0,
                    p: 0.5,
                    t: x,
                })
                .collect(),
            couleur: Couleur::ROUGE,
            epaisseur: 6.0,
            surligneur: false,
        };
        l.pages[0].elements.push(Element::Encre(t));
        let sortie = crate::export::pdf_lecon(&l, dir.path()).unwrap();
        // Relu par hayro : 2 pages, le carré noir du sujet et le trait rouge sur la première.
        let relu = pdf(&sortie, |_, _| {}).unwrap();
        assert_eq!(relu.pages.len(), 2);
        let p1 = tiny_skia::Pixmap::decode_png(&relu.pages[0].png).unwrap();
        let px = |x: f32, y: f32| p1.pixel((x * 2.0) as u32, (y * 2.0) as u32).unwrap();
        assert!(
            px(100.0, 100.0).red() < 40 && px(100.0, 100.0).green() < 40,
            "le sujet est dans l'export"
        );
        let r = px(300.0, 700.0);
        assert!(
            r.red() > 150 && r.green() < 100,
            "l'écriture est dans l'export : {r:?}"
        );
    }
}
