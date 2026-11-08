//! La scène d'une page : une liste de primitives simples, dessinée à l'identique par l'écran (egui)
//! et par l'export (SVG → PNG, PDF). Un seul endroit décide de l'aspect d'une page.

use fano_core::{Couleur, Element, Fond, Format, Forme, FormeKind, Point, Rect, Texte, Trait};

use crate::maths::{self, Morceau};
use crate::polices::{Police, polices};
use crate::rough::{self, Alea};

/// Couleurs du cahier (docs/design.md).
pub mod teinte {
    use fano_core::Couleur;
    pub const PAPIER: Couleur = Couleur([0xfb, 0xfc, 0xfe, 0xff]);
    pub const LIGNE: Couleur = Couleur([0xb9, 0xcb, 0xee, 0xff]);
    pub const LIGNE_FINE: Couleur = Couleur([0xdd, 0xe6, 0xf7, 0xff]);
    pub const MARGE: Couleur = Couleur([0xd8, 0x46, 0x3f, 0xff]);
}

#[derive(Clone, Debug, PartialEq)]
pub enum Prim {
    /// Un polygone plein (contour d'un trait d'encre).
    Remplir { pts: Vec<Point>, couleur: Couleur },
    /// Une ligne brisée tracée.
    Trace {
        pts: Vec<Point>,
        epaisseur: f32,
        couleur: Couleur,
    },
    /// Du texte : `pos` est le début de la ligne de base.
    Texte {
        pos: Point,
        texte: String,
        taille: f32,
        couleur: Couleur,
        police: Police,
    },
    /// Une image du coffre.
    Image { fichier: String, rect: Rect },
}

/// Le fond (papier, lignes, marge) d'une page de taille `w × h`.
pub fn fond(fond: Fond, w: f32, h: f32) -> Vec<Prim> {
    let mut v = Vec::new();
    v.push(Prim::Remplir {
        pts: vec![
            Point::new(0.0, 0.0),
            Point::new(w, 0.0),
            Point::new(w, h),
            Point::new(0.0, h),
        ],
        couleur: teinte::PAPIER,
    });
    let marge_x = 56.0;
    match fond {
        Fond::Blanc => {}
        Fond::Lignes => {
            // Seyès simplifié : une ligne forte tous les 32 pt, trois fines entre.
            let mut y = 96.0;
            let mut i = 0;
            while y < h - 24.0 {
                let forte = i % 4 == 0;
                v.push(Prim::Trace {
                    pts: vec![Point::new(0.0, y), Point::new(w, y)],
                    epaisseur: if forte { 0.6 } else { 0.35 },
                    couleur: if forte { teinte::LIGNE } else { teinte::LIGNE_FINE },
                });
                y += 8.0;
                i += 1;
            }
        }
        Fond::Carreaux => {
            let mut x = 20.0;
            while x < w {
                v.push(Prim::Trace {
                    pts: vec![Point::new(x, 0.0), Point::new(x, h)],
                    epaisseur: 0.4,
                    couleur: teinte::LIGNE_FINE,
                });
                x += 20.0;
            }
            let mut y = 20.0;
            while y < h {
                v.push(Prim::Trace {
                    pts: vec![Point::new(0.0, y), Point::new(w, y)],
                    epaisseur: 0.4,
                    couleur: teinte::LIGNE_FINE,
                });
                y += 20.0;
            }
        }
        Fond::Points => {
            let mut y = 20.0;
            while y < h {
                let mut x = 20.0;
                while x < w {
                    v.push(Prim::Remplir {
                        pts: carre(Point::new(x, y), 0.8),
                        couleur: teinte::LIGNE,
                    });
                    x += 20.0;
                }
                y += 20.0;
            }
        }
    }
    if fond == Fond::Lignes {
        v.push(Prim::Trace {
            pts: vec![Point::new(marge_x, 0.0), Point::new(marge_x, h)],
            epaisseur: 0.8,
            couleur: teinte::MARGE,
        });
    }
    v
}

fn carre(c: Point, r: f32) -> Vec<Point> {
    vec![
        Point::new(c.x - r, c.y - r),
        Point::new(c.x + r, c.y - r),
        Point::new(c.x + r, c.y + r),
        Point::new(c.x - r, c.y + r),
    ]
}

/// Un trait d'encre.
pub fn encre(t: &Trait) -> Prim {
    let reel = t.points.iter().any(|p| (p.p - t.points[0].p).abs() > 1e-3);
    let mut couleur = t.couleur;
    if t.surligneur {
        couleur.0[3] = couleur.0[3].min(0x66);
    }
    Prim::Remplir {
        pts: fano_ink::contour(&t.points, t.epaisseur, reel),
        couleur,
    }
}

/// Une forme au propre, au trait fait main.
pub fn forme(f: &Forme) -> Vec<Prim> {
    let mut alea = Alea::new(f.graine);
    let mut lignes: Vec<Vec<Point>> = Vec::new();
    let ep = f.epaisseur;
    let mut prims = Vec::new();
    let ferme = |pts: &[Point], alea: &mut Alea, lignes: &mut Vec<Vec<Point>>| {
        for i in 0..pts.len() {
            lignes.extend(rough::segment(pts[i], pts[(i + 1) % pts.len()], alea, 1.0));
        }
    };
    match f.kind {
        FormeKind::Segment | FormeKind::Fleche | FormeKind::DoubleFleche | FormeKind::Polyligne => {
            for w in f.points.windows(2) {
                lignes.extend(rough::segment(w[0], w[1], &mut alea, 1.0));
            }
            let n = f.points.len();
            if n >= 2 && matches!(f.kind, FormeKind::Fleche | FormeKind::DoubleFleche) {
                lignes.extend(rough::pointe(f.points[n - 2], f.points[n - 1], ep * 5.0 + 8.0));
            }
            if n >= 2 && f.kind == FormeKind::DoubleFleche {
                lignes.extend(rough::pointe(f.points[1], f.points[0], ep * 5.0 + 8.0));
            }
        }
        FormeKind::Rectangle | FormeKind::Losange | FormeKind::Triangle | FormeKind::Polygone => {
            if f.rempli {
                prims.push(Prim::Remplir {
                    pts: f.points.clone(),
                    couleur: voile(f.couleur),
                });
            }
            ferme(&f.points, &mut alea, &mut lignes);
        }
        FormeKind::Ellipse | FormeKind::Noeud => {
            if let [c, r, ..] = f.points[..] {
                if f.rempli || f.kind == FormeKind::Noeud {
                    let pts = (0..48)
                        .map(|i| {
                            let a = i as f32 / 48.0 * std::f32::consts::TAU;
                            Point::new(c.x + r.x * a.cos(), c.y + r.y * a.sin())
                        })
                        .collect();
                    prims.push(Prim::Remplir {
                        pts,
                        couleur: voile(f.couleur),
                    });
                }
                lignes.extend(rough::ellipse(c, r.x, r.y, &mut alea, 1.0));
            }
        }
    }
    prims.extend(lignes.into_iter().map(|pts| Prim::Trace {
        pts,
        epaisseur: ep,
        couleur: f.couleur,
    }));
    if !f.etiquette.is_empty() {
        let b = f.bbox();
        let taille = 22.0;
        let w = polices().largeur(&f.etiquette, taille);
        let c = b.center();
        prims.extend(texte_ligne(
            &f.etiquette,
            Point::new(c.x - w / 2.0, c.y + taille * 0.3),
            taille,
            f.couleur,
        ));
    }
    prims
}

fn voile(c: Couleur) -> Couleur {
    Couleur([c.0[0], c.0[1], c.0[2], 0x26])
}

/// Une ligne de texte simple (sans formule).
fn texte_ligne(s: &str, base: Point, taille: f32, couleur: Couleur) -> Vec<Prim> {
    let p = polices();
    let mut x = base.x;
    p.morceaux(s)
        .into_iter()
        .map(|(m, police)| {
            let w = p.largeur_police(&m, police, taille);
            let prim = Prim::Texte {
                pos: Point::new(x, base.y),
                texte: m,
                taille,
                couleur,
                police,
            };
            x += w;
            prim
        })
        .collect()
}

/// Taille effective d'un texte selon son niveau de titre.
pub fn taille_titre(t: &Texte) -> f32 {
    t.taille
        * match t.titre {
            1 => 1.7,
            2 => 1.35,
            3 => 1.15,
            _ => 1.0,
        }
}

/// Un morceau d'une ligne de texte : des mots, ou une formule.
enum Bout {
    Mot(String),
    Formule(maths::Boite),
}

/// Met en page un texte au propre ; rend les primitives et le rectangle occupé.
pub fn texte(t: &Texte) -> (Vec<Prim>, Rect) {
    let taille = taille_titre(t);
    let p = polices();
    let interligne = taille * 1.3;
    let espace = p.largeur(" ", taille);
    let mut prims = Vec::new();
    let mut y = t.pos.y + taille * 0.95;
    let mut largeur_max: f32 = 0.0;
    for paragraphe in t.contenu.split('\n') {
        // Découpe en mots et formules.
        let mut bouts: Vec<Bout> = Vec::new();
        for (i, morceau) in paragraphe.split('$').enumerate() {
            if i % 2 == 1 {
                bouts.push(Bout::Formule(maths::mettre_en_page(morceau, taille)));
            } else {
                bouts.extend(
                    morceau
                        .split(' ')
                        .filter(|m| !m.is_empty())
                        .map(|m| Bout::Mot(m.to_owned())),
                );
            }
        }
        let mut x = 0.0;
        let mut haut_ligne: f32 = 0.0;
        let mut bas_ligne: f32 = 0.0;
        let mut ligne: Vec<(f32, &Bout)> = Vec::new();
        let vider = |ligne: &mut Vec<(f32, &Bout)>, y: f32, prims: &mut Vec<Prim>| {
            for (bx, b) in ligne.drain(..) {
                match b {
                    Bout::Mot(m) => {
                        prims.extend(texte_ligne(m, Point::new(t.pos.x + bx, y), taille, t.couleur))
                    }
                    Bout::Formule(boite) => {
                        prims.extend(formule(boite, Point::new(t.pos.x + bx, y), t.couleur))
                    }
                }
            }
        };
        for b in &bouts {
            let (w, h, d) = match b {
                Bout::Mot(m) => (p.largeur(m, taille), taille * 0.75, taille * 0.25),
                Bout::Formule(f) => (f.largeur, f.haut, f.bas),
            };
            if t.largeur > 0.0 && x > 0.0 && x + w > t.largeur {
                y += haut_ligne.max(taille * 0.75) - taille * 0.75;
                vider(&mut ligne, y, &mut prims);
                largeur_max = largeur_max.max(x - espace);
                y += interligne + (bas_ligne - taille * 0.25).max(0.0);
                x = 0.0;
                haut_ligne = 0.0;
                bas_ligne = 0.0;
            }
            ligne.push((x, b));
            x += w + espace;
            haut_ligne = haut_ligne.max(h);
            bas_ligne = bas_ligne.max(d);
        }
        y += haut_ligne.max(taille * 0.75) - taille * 0.75;
        vider(&mut ligne, y, &mut prims);
        largeur_max = largeur_max.max(x - espace);
        y += interligne + (bas_ligne - taille * 0.25).max(0.0);
    }
    if t.titre == 1 && largeur_max > 0.0 {
        // Le titre de leçon est souligné à la main, deux fois.
        let yb = t.pos.y + taille * 1.2;
        let mut alea = Alea::new(t.contenu.len() as u32 + 11);
        for pts in rough::segment(
            Point::new(t.pos.x, yb),
            Point::new(t.pos.x + largeur_max, yb + 1.5),
            &mut alea,
            1.4,
        ) {
            prims.push(Prim::Trace {
                pts,
                epaisseur: 1.6,
                couleur: t.couleur,
            });
        }
    }
    let rect = Rect::from_min_size(
        t.pos,
        largeur_max.max(taille * 0.5),
        (y - interligne + taille * 0.35 - t.pos.y).max(taille),
    );
    if t.encadre != fano_core::Encadre::Aucun {
        prims.extend(encadre(t, rect));
    }
    (prims, rect)
}

/// Le nom et la couleur d'un encadré.
pub fn style_encadre(e: fano_core::Encadre) -> (&'static str, Couleur) {
    use fano_core::Encadre as E;
    match e {
        E::Aucun => ("", Couleur::ENCRE),
        E::Definition => ("Définition", Couleur::BLEU),
        E::ARetenir => ("À retenir", teinte::MARGE),
        E::Exemple => ("Exemple", Couleur::VERT),
        E::Exercice => ("Exercice", Couleur::ORANGE),
        E::Citation => ("Citation", Couleur::VIOLET),
    }
}

/// Le cadre fait main d'un encadré et son étiquette, autour du texte.
fn encadre(t: &Texte, rect: Rect) -> Vec<Prim> {
    let (nom, c) = style_encadre(t.encadre);
    let taille = t.taille * 0.8;
    let r = rect.expand(t.taille * 0.45);
    let r = Rect {
        min: Point::new(r.min.x, r.min.y - taille * 0.9),
        max: r.max,
    };
    let mut alea = Alea::new(t.contenu.len() as u32 * 7 + 3);
    let mut v = Vec::new();
    if t.encadre == fano_core::Encadre::Citation {
        for pts in rough::segment(r.min, Point::new(r.min.x, r.max.y), &mut alea, 1.0) {
            v.push(Prim::Trace {
                pts,
                epaisseur: 2.2,
                couleur: c,
            });
        }
    } else {
        v.push(Prim::Remplir {
            pts: vec![
                r.min,
                Point::new(r.max.x, r.min.y),
                r.max,
                Point::new(r.min.x, r.max.y),
            ],
            couleur: Couleur([c.0[0], c.0[1], c.0[2], 0x10]),
        });
        let coins = [
            r.min,
            Point::new(r.max.x, r.min.y),
            r.max,
            Point::new(r.min.x, r.max.y),
        ];
        for i in 0..4 {
            for pts in rough::segment(coins[i], coins[(i + 1) % 4], &mut alea, 0.8) {
                v.push(Prim::Trace {
                    pts,
                    epaisseur: 1.3,
                    couleur: c,
                });
            }
        }
    }
    v.extend(texte_ligne(
        nom,
        Point::new(r.min.x + t.taille * 0.35, r.min.y + taille * 1.0),
        taille,
        c,
    ));
    v
}

fn formule(b: &maths::Boite, base: Point, couleur: Couleur) -> Vec<Prim> {
    b.morceaux
        .iter()
        .map(|m| match m {
            Morceau::Texte {
                x,
                y,
                texte,
                taille,
                police,
            } => Prim::Texte {
                pos: Point::new(base.x + x, base.y + y),
                texte: texte.clone(),
                taille: *taille,
                couleur,
                police: *police,
            },
            Morceau::Trace { pts, epaisseur } => Prim::Trace {
                pts: pts.iter().map(|p| *p + base).collect(),
                epaisseur: *epaisseur,
                couleur,
            },
        })
        .collect()
}

/// Le rectangle exact d'un élément (le texte est mis en page).
pub fn bbox(el: &Element) -> Rect {
    match el {
        Element::Texte(t) => texte(t).1,
        e => e.bbox(),
    }
}

/// Toutes les primitives d'un élément.
pub fn element(el: &Element) -> Vec<Prim> {
    match el {
        Element::Encre(t) => vec![encre(t)],
        Element::Forme(f) => forme(f),
        Element::Texte(t) => texte(t).0,
        Element::Image(i) => vec![Prim::Image {
            fichier: i.fichier.clone(),
            rect: i.rect,
        }],
    }
}

/// La page entière : fond, surlignages, puis le reste dans l'ordre.
pub fn page(page: &fano_core::Page, format: Format) -> Vec<Prim> {
    let (w, h) = format.taille();
    let mut v = fond(page.fond, w, h);
    let surligne = |e: &&Element| matches!(e, Element::Encre(t) if t.surligneur);
    for e in page.elements.iter().filter(surligne) {
        v.extend(element(e));
    }
    for e in page.elements.iter().filter(|e| !surligne(e)) {
        v.extend(element(e));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(contenu: &str, largeur: f32) -> Texte {
        Texte {
            pos: Point::new(10.0, 10.0),
            contenu: contenu.into(),
            taille: 20.0,
            couleur: Couleur::ENCRE,
            largeur,
            titre: 0,
            encadre: fano_core::Encadre::Aucun,
            source: vec![],
            alternatives: vec![],
        }
    }

    #[test]
    fn retour_a_la_ligne() {
        let (_, une) = texte(&t("un deux trois quatre cinq six sept", 0.0));
        let (_, plusieurs) = texte(&t("un deux trois quatre cinq six sept", 120.0));
        assert!(plusieurs.height() > une.height() * 2.0);
        assert!(plusieurs.width() <= 120.0 + 1.0);
    }

    #[test]
    fn texte_et_formule() {
        let (prims, _) = texte(&t("Soit $x^2$ un carré", 0.0));
        let tout: String = prims
            .iter()
            .filter_map(|p| {
                if let Prim::Texte { texte, .. } = p {
                    Some(texte.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(tout, "Soitx2uncarré");
    }
}
