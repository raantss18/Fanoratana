//! La scène d'une page : une liste de primitives simples, dessinée à l'identique par l'écran (egui)
//! et par l'export (SVG → PNG, PDF). Un seul endroit décide de l'aspect d'une page.

use fano_core::{Couleur, Element, Fond, Format, Forme, FormeKind, Point, Rect, Texte, Trait};

use crate::maths::{self, Morceau};
use crate::polices::{Police, polices};
use crate::rough::{self, Alea};

/// Couleurs du cahier (docs/design.md).
pub mod teinte {
    use fano_core::Couleur;
    pub const PAPIER: Couleur = Couleur([0xff, 0xff, 0xff, 0xff]);
    pub const LIGNE: Couleur = Couleur([0xcf, 0xcf, 0xcf, 0xff]);
    pub const LIGNE_FINE: Couleur = Couleur([0xe8, 0xe8, 0xe8, 0xff]);
    pub const MARGE: Couleur = Couleur([0xbd, 0xbd, 0xbd, 0xff]);
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
/// Listes (une ligne = un élément), alignement, souligné et surligné compris.
pub fn texte(t: &Texte) -> (Vec<Prim>, Rect) {
    use fano_core::{Alignement, Liste};
    let taille = taille_titre(t);
    let p = polices();
    let interligne = taille * 1.3;
    let espace = p.largeur(" ", taille);
    let mut prims = Vec::new();
    let mut fonds = Vec::new();
    let mut y = t.pos.y + taille * 0.95;
    let mut largeur_max: f32 = 0.0;
    let retrait = if t.liste == Liste::Aucune {
        0.0
    } else {
        taille * 1.2
    };
    let dispo = if t.largeur > 0.0 { t.largeur - retrait } else { 0.0 };
    let mut numero = 0;
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
        if t.liste != Liste::Aucune && !paragraphe.trim().is_empty() {
            numero += 1;
            let marque = if t.liste == Liste::Puces {
                "•".to_owned()
            } else {
                format!("{numero}.")
            };
            prims.extend(texte_ligne(&marque, Point::new(t.pos.x, y), taille, t.couleur));
        }
        let mut x = 0.0;
        let mut haut_ligne: f32 = 0.0;
        let mut bas_ligne: f32 = 0.0;
        let mut ligne: Vec<(f32, &Bout)> = Vec::new();
        let vider = |ligne: &mut Vec<(f32, &Bout)>,
                     y: f32,
                     lw: f32,
                     prims: &mut Vec<Prim>,
                     fonds: &mut Vec<Prim>| {
            let decal = retrait
                + match t.alignement {
                    Alignement::Gauche => 0.0,
                    Alignement::Centre => ((dispo - lw) / 2.0).max(0.0),
                    Alignement::Droite => (dispo - lw).max(0.0),
                };
            let x0 = t.pos.x + decal;
            if lw > 0.0 && t.surligne {
                fonds.push(Prim::Remplir {
                    pts: vec![
                        Point::new(x0 - 3.0, y - taille * 0.78),
                        Point::new(x0 + lw + 3.0, y - taille * 0.74),
                        Point::new(x0 + lw + 3.0, y + taille * 0.24),
                        Point::new(x0 - 3.0, y + taille * 0.22),
                    ],
                    couleur: Couleur([0xff, 0xe0, 0x66, 0x99]),
                });
            }
            if lw > 0.0 && t.souligne {
                let mut alea = Alea::new((y as u32).wrapping_add(lw as u32));
                if let Some(pts) = rough::segment(
                    Point::new(x0, y + taille * 0.16),
                    Point::new(x0 + lw, y + taille * 0.17),
                    &mut alea,
                    0.6,
                )
                .into_iter()
                .next()
                {
                    prims.push(Prim::Trace {
                        pts,
                        epaisseur: (taille * 0.06).max(1.0),
                        couleur: t.couleur,
                    });
                }
            }
            for (bx, b) in ligne.drain(..) {
                match b {
                    Bout::Mot(m) => prims.extend(texte_ligne(m, Point::new(x0 + bx, y), taille, t.couleur)),
                    Bout::Formule(boite) => prims.extend(formule(boite, Point::new(x0 + bx, y), t.couleur)),
                }
            }
        };
        for b in &bouts {
            let (w, h, d) = match b {
                Bout::Mot(m) => (p.largeur(m, taille), taille * 0.75, taille * 0.25),
                Bout::Formule(f) => (f.largeur, f.haut, f.bas),
            };
            if dispo > 0.0 && x > 0.0 && x + w > dispo {
                y += haut_ligne.max(taille * 0.75) - taille * 0.75;
                vider(&mut ligne, y, x - espace, &mut prims, &mut fonds);
                largeur_max = largeur_max.max(x - espace + retrait);
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
        vider(&mut ligne, y, (x - espace).max(0.0), &mut prims, &mut fonds);
        largeur_max = largeur_max.max(x - espace + retrait);
        y += interligne + (bas_ligne - taille * 0.25).max(0.0);
    }
    if t.alignement != fano_core::Alignement::Gauche && dispo > 0.0 {
        largeur_max = largeur_max.max(dispo + retrait);
    }
    fonds.append(&mut prims);
    let mut prims = fonds;
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
        // Monochrome : l'étiquette et la forme du cadre les distinguent (docs/design.md).
        E::Aucun => ("", Couleur::NOIR),
        E::Definition => ("Définition", Couleur::NOIR),
        E::ARetenir => ("À retenir", Couleur::NOIR),
        E::Exemple => ("Exemple", Couleur::NOIR),
        E::Exercice => ("Exercice", Couleur::NOIR),
        E::Citation => ("Citation", Couleur::NOIR),
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
            couleur: Couleur([
                c.0[0],
                c.0[1],
                c.0[2],
                if t.encadre == fano_core::Encadre::ARetenir {
                    0x0d
                } else {
                    0x00
                },
            ]),
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
        Element::Tableau(t) => tableau(t),
        Element::Graphe(g) => graphe(g),
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
            ..Default::default()
        }
    }

    #[test]
    fn liste_et_graphe() {
        let mut tx = t("un\ndeux", 0.0);
        tx.liste = fano_core::Liste::Numeros;
        let (prims, _) = texte(&tx);
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
        assert_eq!(tout, "1.un2.deux");
        let defs = vec!["f(x)=x^2".to_owned(), "A=(1;1)".to_owned()];
        let (x0, x1, y0, y1) = bornes_auto(&defs);
        assert!(
            x0 <= -5.0 && x1 >= 5.0 && y0 < 0.0 && y1 >= 20.0,
            "{x0} {x1} {y0} {y1}"
        );
        let g = fano_core::Graphe {
            rect: Rect::from_min_size(Point::new(0.0, 0.0), 300.0, 200.0),
            definitions: defs,
            xmin: x0,
            xmax: x1,
            ymin: y0,
            ymax: y1,
        };
        let prims = graphe(&g);
        assert!(
            prims
                .iter()
                .any(|p| matches!(p, Prim::Trace { pts, .. } if pts.len() > 100)),
            "la parabole est tracée"
        );
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

/// Un tableau : grille au trait fait main, en-tête grisé, texte au propre dans chaque case.
pub fn tableau(t: &fano_core::Tableau) -> Vec<Prim> {
    let mut v = Vec::new();
    let b = t.bbox();
    if t.entete && !t.hauteurs.is_empty() {
        let h = t.hauteurs[0];
        v.push(Prim::Remplir {
            pts: vec![
                b.min,
                Point::new(b.max.x, b.min.y),
                Point::new(b.max.x, b.min.y + h),
                Point::new(b.min.x, b.min.y + h),
            ],
            couleur: Couleur([0, 0, 0, 0x12]),
        });
    }
    let mut alea = Alea::new((t.largeurs.len() * 31 + t.hauteurs.len()) as u32);
    let mut y = t.pos.y;
    for i in 0..=t.hauteurs.len() {
        if let Some(pts) = rough::segment(Point::new(b.min.x, y), Point::new(b.max.x, y), &mut alea, 0.5)
            .into_iter()
            .next()
        {
            v.push(Prim::Trace {
                pts,
                epaisseur: if i == 0 || i == t.hauteurs.len() { 1.6 } else { 1.1 },
                couleur: t.couleur,
            });
        }
        y += t.hauteurs.get(i).copied().unwrap_or(0.0);
    }
    let mut x = t.pos.x;
    for j in 0..=t.largeurs.len() {
        if let Some(pts) = rough::segment(Point::new(x, b.min.y), Point::new(x, b.max.y), &mut alea, 0.5)
            .into_iter()
            .next()
        {
            v.push(Prim::Trace {
                pts,
                epaisseur: if j == 0 || j == t.largeurs.len() { 1.6 } else { 1.1 },
                couleur: t.couleur,
            });
        }
        x += t.largeurs.get(j).copied().unwrap_or(0.0);
    }
    for (l, ligne) in t.cellules.iter().enumerate() {
        for (c, contenu) in ligne.iter().enumerate() {
            if contenu.is_empty() || c >= t.largeurs.len() || l >= t.hauteurs.len() {
                continue;
            }
            let r = t.rect_case(l, c);
            let taille = t.taille.min(r.height() * 0.62);
            let tx = Texte {
                pos: Point::new(r.min.x + 6.0, r.min.y + (r.height() - taille * 1.25) / 2.0),
                contenu: contenu.clone(),
                taille,
                couleur: t.couleur,
                largeur: r.width() - 12.0,
                ..Default::default()
            };
            v.extend(texte(&tx).0);
        }
    }
    v
}

/// Un repère façon GeoGebra : grille, axes gradués, courbes et points.
pub fn graphe(g: &fano_core::Graphe) -> Vec<Prim> {
    use crate::expr::{Definition, definition, evaluer};
    let r = g.rect;
    let (dx, dy) = ((g.xmax - g.xmin).max(1e-3), (g.ymax - g.ymin).max(1e-3));
    let ecran = |x: f64, y: f64| {
        Point::new(
            r.min.x + (x as f32 - g.xmin) / dx * r.width(),
            r.max.y - (y as f32 - g.ymin) / dy * r.height(),
        )
    };
    let mut v = vec![Prim::Remplir {
        pts: vec![
            r.min,
            Point::new(r.max.x, r.min.y),
            r.max,
            Point::new(r.min.x, r.max.y),
        ],
        couleur: Couleur([0xff, 0xff, 0xff, 0xff]),
    }];
    let pas = |etendue: f32| {
        let brut = etendue / 8.0;
        let p10 = 10f32.powf(brut.log10().floor());
        [1.0, 2.0, 5.0, 10.0]
            .into_iter()
            .map(|k| k * p10)
            .find(|s| *s >= brut)
            .unwrap_or(p10 * 10.0)
    };
    let (px, py) = (pas(dx), pas(dy));
    let grille = Couleur([0, 0, 0, 0x18]);
    let mut x = (g.xmin / px).ceil() * px;
    while x <= g.xmax + 1e-4 {
        v.push(Prim::Trace {
            pts: vec![ecran(x as f64, g.ymin as f64), ecran(x as f64, g.ymax as f64)],
            epaisseur: 0.5,
            couleur: grille,
        });
        if x.abs() > 1e-4 && (0.0..=1.0).contains(&((0.0 - g.ymin) / dy)) {
            let q = ecran(x as f64, 0.0);
            v.extend(texte_ligne(
                &format_nombre(x),
                Point::new(q.x - 4.0, q.y + 14.0),
                12.0,
                Couleur([0x5c, 0x5c, 0x5c, 0xff]),
            ));
        }
        x += px;
    }
    let mut y = (g.ymin / py).ceil() * py;
    while y <= g.ymax + 1e-4 {
        v.push(Prim::Trace {
            pts: vec![ecran(g.xmin as f64, y as f64), ecran(g.xmax as f64, y as f64)],
            epaisseur: 0.5,
            couleur: grille,
        });
        if y.abs() > 1e-4 && (0.0..=1.0).contains(&((0.0 - g.xmin) / dx)) {
            let q = ecran(0.0, y as f64);
            v.extend(texte_ligne(
                &format_nombre(y),
                Point::new(q.x + 4.0, q.y + 4.0),
                12.0,
                Couleur([0x5c, 0x5c, 0x5c, 0xff]),
            ));
        }
        y += py;
    }
    // Axes avec flèches.
    if g.ymin <= 0.0 && g.ymax >= 0.0 {
        let (a, b) = (ecran(g.xmin as f64, 0.0), ecran(g.xmax as f64, 0.0));
        v.push(Prim::Trace {
            pts: vec![a, b],
            epaisseur: 1.2,
            couleur: Couleur::NOIR,
        });
        v.extend(rough::pointe(a, b, 8.0).map(|pts| Prim::Trace {
            pts,
            epaisseur: 1.2,
            couleur: Couleur::NOIR,
        }));
    }
    if g.xmin <= 0.0 && g.xmax >= 0.0 {
        let (a, b) = (ecran(0.0, g.ymin as f64), ecran(0.0, g.ymax as f64));
        v.push(Prim::Trace {
            pts: vec![a, b],
            epaisseur: 1.2,
            couleur: Couleur::NOIR,
        });
        v.extend(rough::pointe(a, b, 8.0).map(|pts| Prim::Trace {
            pts,
            epaisseur: 1.2,
            couleur: Couleur::NOIR,
        }));
    }
    let couleurs = [
        Couleur::NOIR,
        Couleur::BLEU,
        Couleur::ROUGE,
        Couleur::VERT,
        Couleur::VIOLET,
    ];
    let mut k = 0;
    for d in &g.definitions {
        match definition(d) {
            Some(Definition::Fonction { nom, expr }) => {
                let c = couleurs[k % couleurs.len()];
                k += 1;
                let n = 400;
                let mut morceau: Vec<Point> = Vec::new();
                let mut dernier: Option<(f64, f64)> = None;
                let (bas, haut) = (g.ymin as f64, g.ymax as f64);
                let dedans = |y: f64| y.is_finite() && y >= bas && y <= haut;
                for i in 0..=n {
                    let xv = g.xmin as f64 + (dx as f64) * i as f64 / n as f64;
                    let yv = evaluer(&expr, xv);
                    // Découpe exacte au bord du cadre : le point où la courbe sort (ou entre).
                    if let Some((x0, y0)) = dernier {
                        let saut = (yv - y0).abs() > dy as f64 * 2.0;
                        if !saut && y0.is_finite() && yv.is_finite() && dedans(y0) != dedans(yv) {
                            let bord = if yv > haut || y0 > haut { haut } else { bas };
                            let t = (bord - y0) / (yv - y0);
                            morceau.push(ecran(x0 + (xv - x0) * t, bord));
                        }
                        if !dedans(yv) || saut {
                            if morceau.len() > 1 {
                                v.push(Prim::Trace {
                                    pts: std::mem::take(&mut morceau),
                                    epaisseur: 2.0,
                                    couleur: c,
                                });
                            }
                            morceau.clear();
                        }
                    }
                    if dedans(yv) {
                        morceau.push(ecran(xv, yv));
                    }
                    dernier = Some((xv, yv));
                }
                if morceau.len() > 1 {
                    v.push(Prim::Trace {
                        pts: morceau,
                        epaisseur: 2.0,
                        couleur: c,
                    });
                }
                // Le nom de la courbe, près du bord droit, là où elle est visible.
                let xe = g.xmax as f64 - dx as f64 * 0.12;
                let ye = evaluer(&expr, xe);
                if ye.is_finite() && ye > g.ymin as f64 && ye < g.ymax as f64 {
                    let q = ecran(xe, ye);
                    let etiquette = if nom == "y" {
                        String::new()
                    } else {
                        format!("𝒞{nom}").replace('𝒞', "C")
                    };
                    v.extend(texte_ligne(&etiquette, Point::new(q.x + 4.0, q.y - 6.0), 16.0, c));
                }
            }
            Some(Definition::Point { nom, x, y }) => {
                let q = ecran(x, y);
                let pts = (0..16)
                    .map(|i| {
                        let a = i as f32 / 16.0 * std::f32::consts::TAU;
                        Point::new(q.x + 3.5 * a.cos(), q.y + 3.5 * a.sin())
                    })
                    .collect();
                v.push(Prim::Remplir {
                    pts,
                    couleur: Couleur::NOIR,
                });
                v.extend(texte_ligne(
                    &nom,
                    Point::new(q.x + 6.0, q.y - 6.0),
                    16.0,
                    Couleur::NOIR,
                ));
            }
            None => {}
        }
    }
    v.push(Prim::Trace {
        pts: vec![
            r.min,
            Point::new(r.max.x, r.min.y),
            r.max,
            Point::new(r.min.x, r.max.y),
            r.min,
        ],
        epaisseur: 0.8,
        couleur: Couleur([0, 0, 0, 0x40]),
    });
    v
}

fn format_nombre(x: f32) -> String {
    let s = format!("{x:.2}");
    s.trim_end_matches('0').trim_end_matches('.').replace('.', ",")
}

/// Un repère adapté aux définitions : x de −5 à 5 (ou autour des points), y d'après les valeurs prises.
pub fn bornes_auto(definitions: &[String]) -> (f32, f32, f32, f32) {
    use crate::expr::{Definition, definition, evaluer};
    let defs: Vec<Definition> = definitions.iter().filter_map(|d| definition(d)).collect();
    let (mut xmin, mut xmax) = (-5.0f64, 5.0f64);
    for d in &defs {
        if let Definition::Point { x, .. } = d {
            xmin = xmin.min(x - 1.0);
            xmax = xmax.max(x + 1.0);
        }
    }
    let mut ys: Vec<f64> = Vec::new();
    for d in &defs {
        match d {
            Definition::Fonction { expr, .. } => {
                for i in 0..=100 {
                    let y = evaluer(expr, xmin + (xmax - xmin) * i as f64 / 100.0);
                    if y.is_finite() {
                        ys.push(y);
                    }
                }
            }
            Definition::Point { y, .. } => ys.push(*y),
        }
    }
    ys.sort_by(f64::total_cmp);
    let (mut ymin, mut ymax) = if ys.is_empty() {
        (-5.0, 5.0)
    } else {
        (ys[ys.len() / 20], ys[ys.len() - 1 - ys.len() / 20])
    };
    ymin = ymin.min(-1.0);
    ymax = ymax.max(1.0);
    let marge = (ymax - ymin) * 0.1;
    (
        xmin as f32,
        xmax as f32,
        (ymin - marge).max(-50.0) as f32,
        (ymax + marge).min(50.0) as f32,
    )
}
