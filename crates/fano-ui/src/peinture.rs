//! Dessiner la scène d'une page avec egui : mêmes primitives que l'export.

use std::collections::HashMap;
use std::path::Path;

use egui::{Color32, FontFamily, FontId, Mesh, Painter, Pos2, Shape, Stroke, TextureHandle, pos2};
use fano_core::{Couleur, Element, Point, Trait};
use fano_render::Prim;
use fano_render::polices::Police;

/// Passage page → écran : `ecran = origine + p × zoom`.
#[derive(Clone, Copy, Debug)]
pub struct Repere {
    pub origine: Pos2,
    pub zoom: f32,
}

impl Repere {
    pub fn ecran(&self, p: Point) -> Pos2 {
        pos2(self.origine.x + p.x * self.zoom, self.origine.y + p.y * self.zoom)
    }
    pub fn page(&self, e: Pos2) -> Point {
        Point::new(
            (e.x - self.origine.x) / self.zoom,
            (e.y - self.origine.y) / self.zoom,
        )
    }
    pub fn rect(&self, r: fano_core::Rect) -> egui::Rect {
        egui::Rect::from_min_max(self.ecran(r.min), self.ecran(r.max))
    }
}

pub fn c32(c: Couleur) -> Color32 {
    let [r, g, b, a] = c.0;
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

pub const MANUSCRIT: &str = "manuscrit";
pub const SYMBOLES: &str = "symboles";

fn famille(p: Police) -> FontFamily {
    match p {
        Police::Manuscrit => FontFamily::Name(MANUSCRIT.into()),
        Police::Symboles => FontFamily::Name(SYMBOLES.into()),
    }
}

/// Les images du coffre, décodées une fois.
#[derive(Default)]
pub struct Images {
    cache: HashMap<String, Option<TextureHandle>>,
}

impl Images {
    pub fn texture(&mut self, ctx: &egui::Context, coffre: &Path, fichier: &str) -> Option<TextureHandle> {
        self.cache
            .entry(fichier.to_owned())
            .or_insert_with(|| {
                let octets = std::fs::read(fano_core::coffre::chemin_media(coffre, fichier)?).ok()?;
                let img = decoder(&octets, fichier)?;
                Some(ctx.load_texture(fichier, img, egui::TextureOptions::LINEAR))
            })
            .clone()
    }
}

/// Décode un PNG, un JPEG ou un SVG (rendu à 2× sa taille, au plus 2048 px).
pub fn decoder(octets: &[u8], nom: &str) -> Option<egui::ColorImage> {
    if nom.to_lowercase().ends_with(".svg") {
        let arbre = usvg::Tree::from_data(octets, &options_svg()).ok()?;
        let s = arbre.size();
        let k = (2048.0 / s.width().max(s.height())).min(2.0);
        let (w, h) = ((s.width() * k).ceil() as u32, (s.height() * k).ceil() as u32);
        let mut pix = tiny_skia::Pixmap::new(w.max(1), h.max(1))?;
        resvg::render(&arbre, tiny_skia::Transform::from_scale(k, k), &mut pix.as_mut());
        return Some(egui::ColorImage::from_rgba_premultiplied(
            [w as usize, h as usize],
            pix.data(),
        ));
    }
    let img = lecteur_image(octets)?.decode().ok()?.to_rgba8();
    let img = if img.width().max(img.height()) > 2048 {
        image::DynamicImage::ImageRgba8(img)
            .thumbnail(2048, 2048)
            .to_rgba8()
    } else {
        img
    };
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [img.width() as usize, img.height() as usize],
        img.as_raw(),
    ))
}

/// Taille naturelle d'une image (largeur, hauteur), pour l'insérer à la bonne proportion.
pub fn taille_image(octets: &[u8], nom: &str) -> Option<(f32, f32)> {
    if nom.to_lowercase().ends_with(".svg") {
        let a = usvg::Tree::from_data(octets, &options_svg()).ok()?;
        return Some((a.size().width(), a.size().height()));
    }
    // Seulement l'en-tête : pas besoin de décoder toute l'image pour connaître sa taille.
    let (w, h) = lecteur_image(octets)?.into_dimensions().ok()?;
    Some((w as f32, h as f32))
}

/// Un lecteur d'image aux limites explicites : au plus 16 000 px de côté et 512 Mo à décoder, pour qu'une
/// image fabriquée (« bombe de décompression ») soit refusée au lieu d'épuiser la mémoire.
fn lecteur_image(octets: &[u8]) -> Option<image::ImageReader<std::io::Cursor<&[u8]>>> {
    let mut l = image::ImageReader::new(std::io::Cursor::new(octets))
        .with_guessed_format()
        .ok()?;
    let mut limites = image::Limits::default();
    limites.max_image_width = Some(16_000);
    limites.max_image_height = Some(16_000);
    limites.max_alloc = Some(512 * 1024 * 1024);
    l.limits(limites);
    Some(l)
}

/// Un SVG inséré ne charge aucun fichier du disque : seules ses images intégrées (`data:`) sont lues.
pub fn options_svg() -> usvg::Options<'static> {
    let mut o = usvg::Options::default();
    o.image_href_resolver = usvg::ImageHrefResolver {
        resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
        resolve_string: Box::new(|_, _| None),
    };
    o
}

/// Un trait d'encre : segments et disques, en un seul maillage (rapide, sans triangulation).
pub fn encre(p: &Painter, r: &Repere, t: &Trait) {
    let reel = t.points.iter().any(|q| (q.p - t.points[0].p).abs() > 1e-3);
    if t.surligneur {
        let pts: Vec<Pos2> = t.points.iter().map(|q| r.ecran(q.pos())).collect();
        let mut c = t.couleur;
        c.0[3] = c.0[3].min(0x66);
        p.add(Shape::line(pts, Stroke::new(t.epaisseur * r.zoom, c32(c))));
        return;
    }
    let sq = fano_ink::squelette(&t.points, t.epaisseur, reel);
    let couleur = c32(t.couleur);
    let mut m = Mesh::default();
    for (i, &(c, rayon)) in sq.iter().enumerate() {
        let ce = r.ecran(c);
        let re = (rayon * r.zoom).max(0.35);
        disque(&mut m, ce, re, couleur);
        if i > 0 {
            let (c0, r0) = sq[i - 1];
            let a = r.ecran(c0);
            let r0 = (r0 * r.zoom).max(0.35);
            let d = (ce - a).normalized();
            let n = egui::vec2(-d.y, d.x);
            let base = m.vertices.len() as u32;
            for q in [a + n * r0, a - n * r0, ce - n * re, ce + n * re] {
                m.colored_vertex(q, couleur);
            }
            m.add_triangle(base, base + 1, base + 2);
            m.add_triangle(base, base + 2, base + 3);
        }
    }
    p.add(Shape::mesh(m));
}

fn disque(m: &mut Mesh, c: Pos2, r: f32, couleur: Color32) {
    let n = ((r * 1.2) as u32).clamp(6, 18);
    let base = m.vertices.len() as u32;
    m.colored_vertex(c, couleur);
    for i in 0..n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        m.colored_vertex(c + egui::vec2(a.cos() * r, a.sin() * r), couleur);
    }
    for i in 0..n {
        m.add_triangle(base, base + 1 + i, base + 1 + (i + 1) % n);
    }
}

/// Une primitive de la scène.
pub fn prim(p: &Painter, r: &Repere, pr: &Prim, images: &mut Images, coffre: &Path) {
    match pr {
        Prim::Remplir { pts, couleur } => {
            let pts: Vec<Pos2> = pts.iter().map(|q| r.ecran(*q)).collect();
            p.add(Shape::convex_polygon(pts, c32(*couleur), Stroke::NONE));
        }
        Prim::Trace {
            pts,
            epaisseur,
            couleur,
        } => {
            let pts: Vec<Pos2> = pts.iter().map(|q| r.ecran(*q)).collect();
            p.add(Shape::line(
                pts,
                Stroke::new((epaisseur * r.zoom).max(0.5), c32(*couleur)),
            ));
        }
        Prim::Texte {
            pos,
            texte,
            taille,
            couleur,
            police,
        } => {
            let px = taille * r.zoom;
            if px < 1.5 {
                return;
            }
            let galley = p.layout_no_wrap(texte.clone(), FontId::new(px, famille(*police)), c32(*couleur));
            // `pos` est sur la ligne de base ; egui place le haut de la ligne.
            let ascent = galley
                .rows
                .first()
                .and_then(|row| row.row.glyphs.first().map(|g| row.pos.y + g.pos.y))
                .unwrap_or(px * 0.8);
            let haut = r.ecran(*pos) - egui::vec2(0.0, ascent);
            p.galley(haut, galley, c32(*couleur));
        }
        Prim::Image { fichier, rect } => {
            let er = r.rect(*rect);
            match images.texture(p.ctx(), coffre, fichier) {
                Some(tex) => {
                    // Garder les proportions dans le rectangle.
                    let [w, h] = tex.size().map(|v| v as f32);
                    let k = (er.width() / w).min(er.height() / h);
                    let taille = egui::vec2(w * k, h * k);
                    let dest = egui::Rect::from_center_size(er.center(), taille);
                    p.image(
                        tex.id(),
                        dest,
                        egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                None => {
                    p.rect_stroke(er, 4.0, Stroke::new(1.0, Color32::GRAY), egui::StrokeKind::Inside);
                }
            }
        }
    }
}

/// Un élément de page.
pub fn element(p: &Painter, r: &Repere, el: &Element, images: &mut Images, coffre: &Path) {
    match el {
        Element::Encre(t) => encre(p, r, t),
        e => {
            for pr in fano_render::scene::element(e) {
                prim(p, r, &pr, images, coffre);
            }
        }
    }
}

/// Une page entière (fond compris).
pub fn page(
    p: &Painter,
    r: &Repere,
    page: &fano_core::Page,
    format: fano_core::Format,
    images: &mut Images,
    coffre: &Path,
) {
    let (w, h) = format.taille();
    // Ombre de la feuille.
    let rect = r.rect(fano_core::Rect::from_min_size(Point::new(0.0, 0.0), w, h));
    p.rect_filled(
        rect.translate(egui::vec2(0.0, 2.0)),
        2.0,
        Color32::from_black_alpha(28),
    );
    for pr in fano_render::scene::fond_de_page(page, w, h) {
        prim(p, r, &pr, images, coffre);
    }
    let surligne = |e: &&Element| matches!(e, Element::Encre(t) if t.surligneur);
    for e in page.elements.iter().filter(surligne) {
        element(p, r, e, images, coffre);
    }
    for e in page.elements.iter().filter(|e| !surligne(e)) {
        element(p, r, e, images, coffre);
    }
}

#[cfg(test)]
mod tests_images {
    use super::*;

    /// Relecture du 2026-11-15 : un SVG inséré pouvait faire lire un fichier du disque (`<image href="/…">`).
    #[test]
    fn un_svg_ne_lit_aucun_fichier_du_disque() {
        let dir = tempfile::tempdir().unwrap();
        let secret = dir.path().join("secret.png");
        let mut rouge = tiny_skia::Pixmap::new(8, 8).unwrap();
        rouge.fill(tiny_skia::Color::from_rgba8(255, 0, 0, 255));
        std::fs::write(&secret, rouge.encode_png().unwrap()).unwrap();
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="8" height="8"><image width="8" height="8" xlink:href="{}"/></svg>"#,
            secret.display()
        );
        let img = decoder(svg.as_bytes(), "piege.svg").unwrap();
        assert!(
            img.pixels.iter().all(|p| p.a() == 0),
            "rien du fichier n'est dessiné"
        );
    }

    #[test]
    fn la_taille_se_lit_sans_decoder_et_le_reste_est_refuse() {
        let png = tiny_skia::Pixmap::new(40, 30).unwrap().encode_png().unwrap();
        assert_eq!(taille_image(&png, "a.png"), Some((40.0, 30.0)));
        assert_eq!(taille_image(b"pas une image", "a.png"), None);
    }
}
