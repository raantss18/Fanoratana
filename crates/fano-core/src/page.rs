//! Une leçon : des pages, chacune avec ses éléments.
//!
//! L'encre reste toujours gardée : un texte ou une forme « au propre » garde les traits dont il
//! vient (`source`), pour revenir à l'encre ou corriger la lecture (esprit de Fikajiana, ADR 0005).

use serde::{Deserialize, Serialize};

use crate::geom::{Point, Rect};

/// Couleur RGBA, 8 bits par canal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Couleur(pub [u8; 4]);

impl Default for Couleur {
    fn default() -> Self {
        Couleur::NOIR
    }
}

impl Couleur {
    pub const NOIR: Couleur = Couleur([0x00, 0x00, 0x00, 0xff]);
    pub const ENCRE: Couleur = Couleur([0x1f, 0x2a, 0x44, 0xff]);
    pub const BLEU: Couleur = Couleur([0x1c, 0x5d, 0xc9, 0xff]);
    pub const ROUGE: Couleur = Couleur([0xc9, 0x2a, 0x2a, 0xff]);
    pub const VERT: Couleur = Couleur([0x2b, 0x8a, 0x3e, 0xff]);
    pub const ORANGE: Couleur = Couleur([0xe8, 0x59, 0x0c, 0xff]);
    pub const VIOLET: Couleur = Couleur([0x70, 0x48, 0xe8, 0xff]);
    pub const SURLIGNEUR: Couleur = Couleur([0xff, 0xd4, 0x3b, 0x66]);

    pub fn hex(self) -> String {
        let [r, g, b, _] = self.0;
        format!("#{r:02x}{g:02x}{b:02x}")
    }
    pub fn alpha(self) -> f32 {
        f32::from(self.0[3]) / 255.0
    }
}

/// Un point d'encre : position (pt de page), pression 0..1, temps en ms depuis le début du trait.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PointEncre {
    pub x: f32,
    pub y: f32,
    #[serde(default = "pression_defaut")]
    pub p: f32,
    #[serde(default)]
    pub t: u32,
}

fn pression_defaut() -> f32 {
    0.5
}

impl PointEncre {
    pub fn pos(&self) -> Point {
        Point::new(self.x, self.y)
    }
}

/// Un trait d'encre tel qu'il a été écrit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Trait {
    pub points: Vec<PointEncre>,
    pub couleur: Couleur,
    /// Épaisseur nominale en pt.
    pub epaisseur: f32,
    /// Surligneur : large, translucide, sous le reste.
    #[serde(default)]
    pub surligneur: bool,
}

impl Trait {
    /// L'épaisseur réelle moyenne du trait (épaisseur du stylo × appui), pour reconnaître le gras.
    pub fn poids(&self) -> f32 {
        if self.points.is_empty() {
            return self.epaisseur;
        }
        let p = self.points.iter().map(|q| q.p).sum::<f32>() / self.points.len() as f32;
        self.epaisseur * (0.45 + 0.9 * p.clamp(0.0, 1.0))
    }

    pub fn bbox(&self) -> Rect {
        Rect::from_points(self.points.iter().map(PointEncre::pos))
    }
    pub fn translate(&mut self, d: Point) {
        for p in &mut self.points {
            p.x += d.x;
            p.y += d.y;
        }
    }
    /// Longueur du chemin.
    pub fn longueur(&self) -> f32 {
        self.points.windows(2).map(|w| w[0].pos().dist(w[1].pos())).sum()
    }
}

/// Un texte au propre (écriture reconnue, ou tapée). `contenu` peut mêler texte et formules entre
/// `$…$` (sous-ensemble de LaTeX : fractions, puissances, indices, racines, lettres grecques).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Texte {
    /// Coin haut-gauche de la première ligne.
    pub pos: Point,
    pub contenu: String,
    /// Hauteur de corps en pt.
    pub taille: f32,
    pub couleur: Couleur,
    /// Largeur maximale avant retour à la ligne (0 = pas de limite).
    #[serde(default)]
    pub largeur: f32,
    /// Style de titre : 0 = texte, 1 = titre de leçon, 2 = section, 3 = sous-section.
    #[serde(default)]
    pub titre: u8,
    /// Un encadré de cours (toutes matières).
    #[serde(default)]
    pub encadre: Encadre,
    /// Liste à puces ou numérotée (une ligne = un élément).
    #[serde(default)]
    pub liste: Liste,
    #[serde(default)]
    pub alignement: Alignement,
    #[serde(default)]
    pub souligne: bool,
    /// Gras : écrit plus épais que d'habitude, ou choisi au clavier.
    #[serde(default)]
    pub gras: bool,
    /// Surligné (fond jaune pâle à l'écran, gris à l'impression monochrome).
    #[serde(default)]
    pub surligne: bool,
    /// L'encre dont il vient.
    #[serde(default)]
    pub source: Vec<Trait>,
    /// Les autres lectures proposées, de la plus probable à la moins probable.
    #[serde(default)]
    pub alternatives: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Liste {
    #[default]
    Aucune,
    Puces,
    Numeros,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignement {
    #[default]
    Gauche,
    Centre,
    Droite,
}

/// Un tableau : colonnes et lignes de tailles libres, du texte (avec formules `$…$`) dans chaque case.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tableau {
    pub pos: Point,
    pub largeurs: Vec<f32>,
    pub hauteurs: Vec<f32>,
    /// `cellules[ligne][colonne]`.
    pub cellules: Vec<Vec<String>>,
    /// La première ligne est un en-tête (fond gris, écriture appuyée).
    #[serde(default)]
    pub entete: bool,
    pub couleur: Couleur,
    pub taille: f32,
}

impl Tableau {
    pub fn nouveau(pos: Point, lignes: usize, colonnes: usize, largeur: f32) -> Tableau {
        let (l, c) = (lignes.max(1), colonnes.max(1));
        Tableau {
            pos,
            largeurs: vec![largeur / c as f32; c],
            hauteurs: vec![40.0; l],
            cellules: vec![vec![String::new(); c]; l],
            entete: true,
            couleur: Couleur::NOIR,
            taille: 20.0,
        }
    }
    pub fn bbox(&self) -> Rect {
        Rect::from_min_size(self.pos, self.largeurs.iter().sum(), self.hauteurs.iter().sum())
    }
    /// La case sous un point, `(ligne, colonne)`.
    pub fn case(&self, p: Point) -> Option<(usize, usize)> {
        if !self.bbox().contains(p) {
            return None;
        }
        let mut y = self.pos.y;
        let l = self.hauteurs.iter().position(|h| {
            y += h;
            p.y <= y
        })?;
        let mut x = self.pos.x;
        let c = self.largeurs.iter().position(|w| {
            x += w;
            p.x <= x
        })?;
        Some((l, c))
    }
    /// Le rectangle d'une case.
    pub fn rect_case(&self, l: usize, c: usize) -> Rect {
        let x = self.pos.x + self.largeurs[..c].iter().sum::<f32>();
        let y = self.pos.y + self.hauteurs[..l].iter().sum::<f32>();
        Rect::from_min_size(Point::new(x, y), self.largeurs[c], self.hauteurs[l])
    }
    pub fn ajouter_ligne(&mut self) {
        let h = self.hauteurs.last().copied().unwrap_or(40.0);
        self.hauteurs.push(h);
        self.cellules.push(vec![String::new(); self.largeurs.len()]);
    }
    pub fn ajouter_colonne(&mut self) {
        let w = self.largeurs.last().copied().unwrap_or(120.0);
        self.largeurs.push(w);
        self.cellules.iter_mut().for_each(|l| l.push(String::new()));
    }
    pub fn retirer_ligne(&mut self) {
        if self.hauteurs.len() > 1 {
            self.hauteurs.pop();
            self.cellules.pop();
        }
    }
    pub fn retirer_colonne(&mut self) {
        if self.largeurs.len() > 1 {
            self.largeurs.pop();
            self.cellules.iter_mut().for_each(|l| {
                l.pop();
            });
        }
    }
}

/// Une courbe façon GeoGebra : des fonctions `f(x)=…` et des points `A=(1,2)`, dans un repère.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Graphe {
    pub rect: Rect,
    /// Les définitions telles qu'écrites : `f(x)=x^2-1`, `A=(1,2)`.
    pub definitions: Vec<String>,
    pub xmin: f32,
    pub xmax: f32,
    pub ymin: f32,
    pub ymax: f32,
}

/// Les encadrés d'un cours, valables pour toutes les matières.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Encadre {
    #[default]
    Aucun,
    Definition,
    ARetenir,
    Exemple,
    Exercice,
    Citation,
}

impl Encadre {
    pub const TOUS: [Encadre; 6] = [
        Encadre::Aucun,
        Encadre::Definition,
        Encadre::ARetenir,
        Encadre::Exemple,
        Encadre::Exercice,
        Encadre::Citation,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormeKind {
    Segment,
    Fleche,
    /// Flèche aux deux bouts.
    DoubleFleche,
    Rectangle,
    Losange,
    Ellipse,
    Triangle,
    /// Polygone fermé quelconque.
    Polygone,
    /// Ligne brisée ouverte.
    Polyligne,
    /// Un nœud de graphe ou d'arbre : un cercle, avec éventuellement un texte dedans.
    Noeud,
}

/// Une forme au propre, dessinée façon Excalidraw (trait légèrement tremblé, graine fixe).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Forme {
    pub kind: FormeKind,
    /// Sommets (polygones, segments) ; pour une ellipse ou un nœud : centre puis rayons (rx, ry).
    pub points: Vec<Point>,
    pub couleur: Couleur,
    pub epaisseur: f32,
    /// Remplissage translucide de la couleur du trait.
    #[serde(default)]
    pub rempli: bool,
    /// Graine du tremblé (le même dessin à chaque affichage).
    pub graine: u32,
    /// Étiquette (nœud, sommet nommé).
    #[serde(default)]
    pub etiquette: String,
    #[serde(default)]
    pub source: Vec<Trait>,
}

impl Forme {
    pub fn bbox(&self) -> Rect {
        match self.kind {
            FormeKind::Ellipse | FormeKind::Noeud if self.points.len() >= 2 => {
                let c = self.points[0];
                let r = self.points[1];
                Rect {
                    min: Point::new(c.x - r.x, c.y - r.y),
                    max: Point::new(c.x + r.x, c.y + r.y),
                }
            }
            _ => Rect::from_points(self.points.iter().copied()),
        }
    }
    pub fn translate(&mut self, d: Point) {
        match self.kind {
            FormeKind::Ellipse | FormeKind::Noeud => {
                if let Some(c) = self.points.first_mut() {
                    *c = *c + d;
                }
            }
            _ => self.points.iter_mut().for_each(|p| *p = *p + d),
        }
        self.source.iter_mut().for_each(|t| t.translate(d));
    }
}

/// Une image insérée (jpg, png, svg), copiée dans le dossier `medias/` du coffre.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageEl {
    /// Chemin relatif au coffre, ex. `medias/schema-1.png`.
    pub fichier: String,
    pub rect: Rect,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Element {
    Encre(Trait),
    Texte(Texte),
    Forme(Forme),
    Image(ImageEl),
    Tableau(Tableau),
    Graphe(Graphe),
}

impl Element {
    /// Le rectangle occupé ; pour un texte, une estimation (la mise en page exacte est au rendu).
    pub fn bbox(&self) -> Rect {
        match self {
            Element::Encre(t) => t.bbox().expand(t.epaisseur),
            Element::Forme(f) => f.bbox().expand(f.epaisseur),
            Element::Image(i) => i.rect,
            Element::Tableau(t) => t.bbox(),
            Element::Graphe(g) => g.rect,
            Element::Texte(t) => {
                let lignes = t.contenu.lines().count().max(1) as f32;
                let plus_longue = t.contenu.lines().map(|l| l.chars().count()).max().unwrap_or(0) as f32;
                let mut w = plus_longue * t.taille * 0.42;
                if t.largeur > 0.0 {
                    w = w.min(t.largeur);
                }
                Rect::from_min_size(t.pos, w.max(t.taille), lignes * t.taille * 1.25)
            }
        }
    }
    pub fn translate(&mut self, d: Point) {
        match self {
            Element::Encre(t) => t.translate(d),
            Element::Forme(f) => f.translate(d),
            Element::Image(i) => i.rect = i.rect.translate(d),
            Element::Tableau(t) => t.pos = t.pos + d,
            Element::Graphe(g) => g.rect = g.rect.translate(d),
            Element::Texte(t) => {
                t.pos = t.pos + d;
                t.source.iter_mut().for_each(|s| s.translate(d));
            }
        }
    }
}

/// Le fond de page, comme dans un cahier.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fond {
    #[default]
    Blanc,
    Lignes,
    Carreaux,
    Points,
}

/// Format des pages d'une leçon.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Format {
    /// A4 portrait, 595 × 842 pt.
    #[default]
    A4,
    /// Diapositive 16:9, 960 × 540 pt.
    Diapo,
}

impl Format {
    pub fn taille(self) -> (f32, f32) {
        match self {
            Format::A4 => (595.0, 842.0),
            Format::Diapo => (960.0, 540.0),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub elements: Vec<Element>,
    #[serde(default)]
    pub fond: Fond,
}

/// Une leçon : le fichier `.fano` (JSON).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lecon {
    /// Version du format de fichier.
    pub format_fichier: u32,
    pub titre: String,
    #[serde(default)]
    pub format: Format,
    pub pages: Vec<Page>,
    /// Étiquettes libres (`#géométrie`), comme dans Obsidian.
    #[serde(default)]
    pub etiquettes: Vec<String>,
}

impl Lecon {
    pub const FORMAT_FICHIER: u32 = 1;

    pub fn nouvelle(titre: &str, format: Format) -> Lecon {
        let fond = if format == Format::A4 {
            Fond::Lignes
        } else {
            Fond::Blanc
        };
        Lecon {
            format_fichier: Self::FORMAT_FICHIER,
            titre: titre.to_owned(),
            format,
            pages: vec![Page {
                elements: Vec::new(),
                fond,
            }],
            etiquettes: Vec::new(),
        }
    }

    pub fn depuis_json(s: &str) -> Result<Lecon, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn vers_json(&self) -> String {
        serde_json::to_string(self).expect("une leçon se sérialise toujours")
    }

    /// Le texte de la leçon, en Markdown (titres, paragraphes, formules `$…$`, images) : pour la
    /// recherche et pour l'export vers Obsidian.
    pub fn markdown(&self) -> String {
        let mut md = format!("# {}\n", self.titre);
        for (i, page) in self.pages.iter().enumerate() {
            if i > 0 {
                md.push_str("\n---\n");
            }
            let mut els: Vec<&Element> = page.elements.iter().collect();
            els.sort_by(|a, b| {
                let (ra, rb) = (a.bbox(), b.bbox());
                ra.min.y.total_cmp(&rb.min.y).then(ra.min.x.total_cmp(&rb.min.x))
            });
            for el in els {
                match el {
                    Element::Texte(t) => {
                        md.push('\n');
                        if t.titre > 0 {
                            md.push_str(&"#".repeat(usize::from(t.titre) + 1));
                            md.push(' ');
                        }
                        md.push_str(&t.contenu);
                        md.push('\n');
                    }
                    Element::Image(im) => md.push_str(&format!("\n![]({})\n", im.fichier)),
                    Element::Tableau(t) => {
                        md.push('\n');
                        for (i, l) in t.cellules.iter().enumerate() {
                            md.push_str(&format!("| {} |\n", l.join(" | ")));
                            if i == 0 {
                                md.push_str(&format!("|{}\n", " --- |".repeat(l.len())));
                            }
                        }
                    }
                    Element::Graphe(g) => {
                        md.push_str(&format!("\n```geogebra\n{}\n```\n", g.definitions.join("\n")));
                    }
                    _ => {}
                }
            }
        }
        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aller_retour_json() {
        let mut l = Lecon::nouvelle("Les fractions", Format::A4);
        l.pages[0].elements.push(Element::Encre(Trait {
            points: vec![
                PointEncre {
                    x: 1.0,
                    y: 2.0,
                    p: 0.4,
                    t: 0,
                },
                PointEncre {
                    x: 3.0,
                    y: 4.0,
                    p: 0.6,
                    t: 16,
                },
            ],
            couleur: Couleur::ENCRE,
            epaisseur: 2.0,
            surligneur: false,
        }));
        l.pages[0].elements.push(Element::Texte(Texte {
            pos: Point::new(40.0, 60.0),
            contenu: "Une fraction $\\frac{a}{b}$".into(),
            taille: 22.0,
            couleur: Couleur::BLEU,
            largeur: 0.0,
            titre: 0,
            encadre: Encadre::Aucun,
            liste: Liste::Aucune,
            alignement: Alignement::Gauche,
            souligne: false,
            gras: true,
            surligne: false,
            source: vec![],
            alternatives: vec![],
        }));
        let s = l.vers_json();
        assert_eq!(Lecon::depuis_json(&s).unwrap(), l);
        assert!(l.markdown().contains("Une fraction $\\frac{a}{b}$"));
    }

    #[test]
    fn poids_du_trait() {
        let pts = |p: f32| {
            (0..10)
                .map(|i| PointEncre {
                    x: i as f32,
                    y: 0.0,
                    p,
                    t: 0,
                })
                .collect::<Vec<_>>()
        };
        let fin = Trait {
            points: pts(0.5),
            couleur: Couleur::NOIR,
            epaisseur: 2.2,
            surligneur: false,
        };
        let appuye = Trait {
            points: pts(1.0),
            ..fin.clone()
        };
        let large = Trait {
            epaisseur: 4.0,
            ..fin.clone()
        };
        assert!(appuye.poids() > fin.poids() * 1.3);
        assert!(
            large.poids() >= fin.poids() * 1.45,
            "un stylo nettement plus large est du gras"
        );
    }

    #[test]
    fn tableau_cases_et_markdown() {
        let mut t = Tableau::nouveau(Point::new(10.0, 10.0), 2, 3, 300.0);
        assert_eq!(t.case(Point::new(15.0, 15.0)), Some((0, 0)));
        assert_eq!(t.case(Point::new(305.0, 85.0)), Some((1, 2)));
        assert_eq!(t.case(Point::new(400.0, 15.0)), None);
        t.ajouter_ligne();
        t.ajouter_colonne();
        assert_eq!((t.cellules.len(), t.cellules[0].len()), (3, 4));
        t.cellules[0][0] = "Date".into();
        let mut l = Lecon::nouvelle("x", Format::A4);
        l.pages[0].elements.push(Element::Tableau(t));
        assert!(
            l.markdown()
                .contains("| Date |  |  |  |\n| --- | --- | --- | --- |")
        );
    }

    #[test]
    fn translation_d_une_ellipse_ne_bouge_que_le_centre() {
        let mut f = Forme {
            kind: FormeKind::Ellipse,
            points: vec![Point::new(10.0, 10.0), Point::new(5.0, 3.0)],
            couleur: Couleur::ENCRE,
            epaisseur: 2.0,
            rempli: false,
            graine: 1,
            etiquette: String::new(),
            source: vec![],
        };
        f.translate(Point::new(1.0, 2.0));
        assert_eq!(f.points, vec![Point::new(11.0, 12.0), Point::new(5.0, 3.0)]);
    }
}
