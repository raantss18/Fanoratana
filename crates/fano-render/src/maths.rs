//! Un petit moteur de formules, écrit pour Fanoratana : le sous-ensemble de LaTeX qu'on écrit dans
//! un cours (fractions, puissances, indices, racines, vecteurs, lettres grecques, symboles usuels),
//! mis en page en boîtes à la manière de TeX, puis écrit avec la police manuscrite.
//!
//! Ce qui n'est pas compris s'écrit tel quel : rien n'est jamais perdu.

use fano_core::Point;

use crate::polices::{Police, polices};

/// Un morceau de formule placé, relatif à la ligne de base (y vers le bas).
#[derive(Clone, Debug, PartialEq)]
pub enum Morceau {
    Texte {
        x: f32,
        y: f32,
        texte: String,
        taille: f32,
        police: Police,
    },
    /// Un trait (barre de fraction, racine, flèche de vecteur).
    Trace { pts: Vec<Point>, epaisseur: f32 },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Boite {
    pub largeur: f32,
    /// Hauteur au-dessus de la ligne de base.
    pub haut: f32,
    /// Profondeur sous la ligne de base.
    pub bas: f32,
    pub morceaux: Vec<Morceau>,
}

impl Boite {
    fn decaler(&mut self, dx: f32, dy: f32) {
        for m in &mut self.morceaux {
            match m {
                Morceau::Texte { x, y, .. } => {
                    *x += dx;
                    *y += dy;
                }
                Morceau::Trace { pts, .. } => pts.iter_mut().for_each(|p| *p = *p + Point::new(dx, dy)),
            }
        }
    }
    fn ajouter(&mut self, mut autre: Boite, dx: f32, dy: f32) {
        autre.decaler(dx, dy);
        self.haut = self.haut.max(autre.haut - dy);
        self.bas = self.bas.max(autre.bas + dy);
        self.morceaux.extend(autre.morceaux);
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Noeud {
    Signe(String),
    /// Un opérateur entouré d'espace (+, =, ≤…).
    Operateur(String),
    /// Un nom de fonction droit (sin, lim…).
    Fonction(String),
    Groupe(Vec<Noeud>),
    Fraction(Box<Noeud>, Box<Noeud>),
    Racine(Box<Noeud>),
    Exposant(Box<Noeud>, Box<Noeud>),
    Indice(Box<Noeud>, Box<Noeud>),
    IndiceExposant(Box<Noeud>, Box<Noeud>, Box<Noeud>),
    Vecteur(Box<Noeud>),
    Barre(Box<Noeud>),
    Espace(f32),
}

/// Le symbole d'une commande `\nom`, et s'il s'écrit comme un opérateur.
fn symbole(nom: &str) -> Option<(&'static str, bool)> {
    Some(match nom {
        "alpha" => ("α", false),
        "beta" => ("β", false),
        "gamma" => ("γ", false),
        "delta" => ("δ", false),
        "epsilon" | "varepsilon" => ("ε", false),
        "zeta" => ("ζ", false),
        "eta" => ("η", false),
        "theta" => ("θ", false),
        "lambda" => ("λ", false),
        "mu" => ("μ", false),
        "nu" => ("ν", false),
        "xi" => ("ξ", false),
        "pi" => ("π", false),
        "rho" => ("ρ", false),
        "sigma" => ("σ", false),
        "tau" => ("τ", false),
        "phi" | "varphi" => ("φ", false),
        "chi" => ("χ", false),
        "psi" => ("ψ", false),
        "omega" => ("ω", false),
        "Gamma" => ("Γ", false),
        "Delta" => ("Δ", false),
        "Theta" => ("Θ", false),
        "Lambda" => ("Λ", false),
        "Pi" => ("Π", false),
        "Sigma" => ("Σ", false),
        "Phi" => ("Φ", false),
        "Omega" => ("Ω", false),
        "infty" => ("∞", false),
        "partial" => ("∂", false),
        "nabla" => ("∇", false),
        "emptyset" | "varnothing" => ("∅", false),
        "ldots" | "dots" | "cdots" => ("…", false),
        "angle" => ("∠", false),
        "degree" | "circ" => ("°", false),
        "sum" => ("∑", false),
        "prod" => ("∏", false),
        "int" => ("∫", false),
        "forall" => ("∀", false),
        "exists" => ("∃", false),
        "neg" | "lnot" => ("¬", false),
        "times" => ("×", true),
        "cdot" => ("·", true),
        "div" => ("÷", true),
        "pm" => ("±", true),
        "mp" => ("∓", true),
        "leq" | "le" => ("≤", true),
        "geq" | "ge" => ("≥", true),
        "neq" | "ne" => ("≠", true),
        "approx" => ("≈", true),
        "equiv" => ("≡", true),
        "sim" => ("∼", true),
        "to" | "rightarrow" => ("→", true),
        "leftarrow" => ("←", true),
        "Rightarrow" | "implies" => ("⇒", true),
        "Leftarrow" => ("⇐", true),
        "Leftrightarrow" | "iff" => ("⇔", true),
        "mapsto" => ("↦", true),
        "in" => ("∈", true),
        "notin" => ("∉", true),
        "subset" => ("⊂", true),
        "subseteq" => ("⊆", true),
        "cap" => ("∩", true),
        "cup" => ("∪", true),
        "wedge" | "land" => ("∧", true),
        "vee" | "lor" => ("∨", true),
        "parallel" => ("∥", true),
        "perp" => ("⊥", true),
        _ => return None,
    })
}

const FONCTIONS: [&str; 16] = [
    "sin", "cos", "tan", "ln", "log", "exp", "lim", "max", "min", "sup", "inf", "det", "arccos", "arcsin",
    "arctan", "pgcd",
];

fn ensemble(c: char) -> Option<&'static str> {
    Some(match c {
        'R' => "ℝ",
        'N' => "ℕ",
        'Z' => "ℤ",
        'Q' => "ℚ",
        'C' => "ℂ",
        _ => return None,
    })
}

struct Lecteur<'a> {
    c: std::iter::Peekable<std::str::Chars<'a>>,
}

impl Lecteur<'_> {
    fn espaces(&mut self) {
        while self.c.peek().is_some_and(|c| *c == ' ') {
            self.c.next();
        }
    }

    fn commande(&mut self) -> String {
        let mut nom = String::new();
        while let Some(&c) = self.c.peek() {
            if c.is_ascii_alphabetic() {
                nom.push(c);
                self.c.next();
            } else {
                break;
            }
        }
        if nom.is_empty() {
            // `\,` `\;` `\{` `\}` `\\`
            if let Some(c) = self.c.next() {
                nom.push(c);
            }
        }
        nom
    }

    /// Un argument : `{…}` ou un seul signe.
    fn argument(&mut self) -> Noeud {
        self.espaces();
        match self.c.peek() {
            Some('{') => {
                self.c.next();
                Noeud::Groupe(self.liste(true))
            }
            Some(_) => self.atome().unwrap_or(Noeud::Groupe(vec![])),
            None => Noeud::Groupe(vec![]),
        }
    }

    fn texte_brut(&mut self) -> String {
        self.espaces();
        let mut s = String::new();
        if self.c.peek() == Some(&'{') {
            self.c.next();
            let mut prof = 1;
            for c in self.c.by_ref() {
                match c {
                    '{' => prof += 1,
                    '}' => {
                        prof -= 1;
                        if prof == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                s.push(c);
            }
        }
        s
    }

    fn atome(&mut self) -> Option<Noeud> {
        self.espaces();
        let c = self.c.next()?;
        Some(match c {
            '{' => Noeud::Groupe(self.liste(true)),
            '\\' => {
                let nom = self.commande();
                match nom.as_str() {
                    "frac" | "dfrac" | "tfrac" => {
                        let n = self.argument();
                        let d = self.argument();
                        Noeud::Fraction(Box::new(n), Box::new(d))
                    }
                    "sqrt" => Noeud::Racine(Box::new(self.argument())),
                    "vec" | "overrightarrow" => Noeud::Vecteur(Box::new(self.argument())),
                    "overline" | "bar" => Noeud::Barre(Box::new(self.argument())),
                    "text" | "mathrm" | "textrm" | "operatorname" => Noeud::Signe(self.texte_brut()),
                    "mathbb" => {
                        let t = self.texte_brut();
                        let mut c = t.chars();
                        match (c.next().and_then(ensemble), c.next()) {
                            (Some(s), None) => Noeud::Signe(s.into()),
                            _ => Noeud::Signe(t),
                        }
                    }
                    "left" | "right" | "displaystyle" | "mathit" | "limits" => Noeud::Groupe(vec![]),
                    "," | ":" | ">" => Noeud::Espace(0.17),
                    ";" => Noeud::Espace(0.28),
                    "quad" => Noeud::Espace(1.0),
                    "qquad" => Noeud::Espace(2.0),
                    "{" => Noeud::Signe("{".into()),
                    "}" => Noeud::Signe("}".into()),
                    "%" => Noeud::Signe("%".into()),
                    n if FONCTIONS.contains(&n) => Noeud::Fonction(n.into()),
                    n => match symbole(n) {
                        Some((s, true)) => Noeud::Operateur(s.into()),
                        Some((s, false)) => Noeud::Signe(s.into()),
                        None => Noeud::Signe(format!("\\{n}")),
                    },
                }
            }
            '+' | '=' | '<' | '>' => Noeud::Operateur(c.to_string()),
            '-' => Noeud::Operateur("−".into()),
            '*' => Noeud::Operateur("×".into()),
            '}' => Noeud::Groupe(vec![]),
            c => Noeud::Signe(c.to_string()),
        })
    }

    fn liste(&mut self, jusqua_accolade: bool) -> Vec<Noeud> {
        let mut res: Vec<Noeud> = Vec::new();
        loop {
            self.espaces();
            match self.c.peek() {
                None => break,
                Some('}') if jusqua_accolade => {
                    self.c.next();
                    break;
                }
                Some('^') | Some('_') => {
                    let haut = self.c.next() == Some('^');
                    let base = res.pop().unwrap_or(Noeud::Groupe(vec![]));
                    let arg = self.argument();
                    let n = match (base, haut) {
                        (Noeud::Indice(b, i), true) => Noeud::IndiceExposant(b, i, Box::new(arg)),
                        (Noeud::Exposant(b, e), false) => Noeud::IndiceExposant(b, Box::new(arg), e),
                        (b, true) => Noeud::Exposant(Box::new(b), Box::new(arg)),
                        (b, false) => Noeud::Indice(Box::new(b), Box::new(arg)),
                    };
                    res.push(n);
                }
                Some(_) => {
                    if let Some(a) = self.atome() {
                        res.push(a);
                    }
                }
            }
        }
        res
    }
}

fn analyser(src: &str) -> Vec<Noeud> {
    Lecteur {
        c: src.chars().peekable(),
    }
    .liste(false)
}

const EP: f32 = 0.055; // épaisseur des traits, en fraction de la taille

fn texte(s: &str, taille: f32) -> Boite {
    let p = polices();
    let mut b = Boite {
        largeur: 0.0,
        haut: taille * 0.72,
        bas: taille * 0.2,
        morceaux: vec![],
    };
    for (m, pol) in p.morceaux(s) {
        let w = p.largeur_police(&m, pol, taille);
        b.morceaux.push(Morceau::Texte {
            x: b.largeur,
            y: 0.0,
            texte: m,
            taille,
            police: pol,
        });
        b.largeur += w;
    }
    b
}

fn horizontal(noeuds: &[Noeud], taille: f32) -> Boite {
    let mut b = Boite {
        largeur: 0.0,
        haut: taille * 0.6,
        bas: taille * 0.15,
        morceaux: vec![],
    };
    for (i, n) in noeuds.iter().enumerate() {
        let operateur = matches!(n, Noeud::Operateur(_));
        // Un « − » en tête (ou après un opérateur) est un signe, sans espace.
        let unaire = operateur && (i == 0 || matches!(noeuds[i - 1], Noeud::Operateur(_)));
        let marge = if operateur && !unaire { taille * 0.2 } else { 0.0 };
        if matches!(n, Noeud::Fonction(_)) && i > 0 {
            b.largeur += taille * 0.12;
        }
        let x = b.largeur + marge;
        let enfant = boite(n, taille);
        b.largeur = x + enfant.largeur + marge;
        if matches!(n, Noeud::Fonction(_)) {
            b.largeur += taille * 0.12;
        }
        b.ajouter(enfant, x, 0.0);
    }
    b
}

fn boite(n: &Noeud, taille: f32) -> Boite {
    let t_petit = (taille * 0.7).max(7.0);
    match n {
        Noeud::Signe(s) | Noeud::Operateur(s) | Noeud::Fonction(s) => texte(s, taille),
        Noeud::Groupe(v) => horizontal(v, taille),
        Noeud::Espace(e) => Boite {
            largeur: taille * e,
            ..Default::default()
        },
        Noeud::Exposant(base, e) => {
            let mut b = boite(base, taille);
            let e = boite(e, t_petit);
            let monte = (b.haut - t_petit * 0.55).max(taille * 0.3);
            let x = b.largeur + taille * 0.01;
            b.largeur = x + e.largeur;
            b.ajouter(e, x, -monte);
            b
        }
        Noeud::Indice(base, i) => {
            let mut b = boite(base, taille);
            let i = boite(i, t_petit);
            let x = b.largeur + taille * 0.02;
            b.largeur = x + i.largeur;
            b.ajouter(i, x, taille * 0.22);
            b
        }
        Noeud::IndiceExposant(base, i, e) => {
            let mut b = boite(base, taille);
            let i = boite(i, t_petit);
            let e = boite(e, t_petit);
            let x = b.largeur + taille * 0.03;
            let monte = (b.haut - t_petit * 0.55).max(taille * 0.3);
            b.largeur = x + i.largeur.max(e.largeur);
            b.ajouter(i, x, taille * 0.24);
            b.ajouter(e, x, -monte);
            b
        }
        Noeud::Fraction(num, den) => {
            let t = (taille * 0.85).max(8.0);
            let num = boite(num, t);
            let den = boite(den, t);
            let larg = num.largeur.max(den.largeur) + taille * 0.25;
            let axe = taille * 0.3;
            let jeu = taille * 0.12;
            let mut b = Boite {
                largeur: larg + taille * 0.1,
                haut: 0.0,
                bas: 0.0,
                morceaux: vec![],
            };
            let (wn, wd) = (num.largeur, den.largeur);
            let yn = -axe - jeu - num.bas;
            let yd = -axe + jeu + den.haut;
            b.ajouter(num, taille * 0.05 + (larg - wn) / 2.0, yn);
            b.ajouter(den, taille * 0.05 + (larg - wd) / 2.0, yd);
            b.morceaux.push(Morceau::Trace {
                pts: vec![
                    Point::new(taille * 0.05, -axe),
                    Point::new(taille * 0.05 + larg, -axe),
                ],
                epaisseur: taille * EP,
            });
            b
        }
        Noeud::Racine(inner) => {
            let int = boite(inner, taille);
            let haut = int.haut + taille * 0.14;
            let bas = int.bas.max(taille * 0.05);
            let x0 = taille * 0.55;
            let mut b = Boite {
                largeur: x0 + int.largeur + taille * 0.1,
                haut,
                bas,
                morceaux: vec![],
            };
            b.morceaux.push(Morceau::Trace {
                pts: vec![
                    Point::new(0.0, -taille * 0.25),
                    Point::new(taille * 0.15, -taille * 0.32),
                    Point::new(taille * 0.32, bas),
                    Point::new(x0 - taille * 0.05, -haut),
                    Point::new(b.largeur, -haut),
                ],
                epaisseur: taille * EP,
            });
            b.ajouter(int, x0, 0.0);
            b
        }
        Noeud::Vecteur(inner) | Noeud::Barre(inner) => {
            let mut b = boite(inner, taille);
            let y = -b.haut - taille * 0.12;
            let (x0, x1) = (taille * 0.05, b.largeur - taille * 0.02);
            b.morceaux.push(Morceau::Trace {
                pts: vec![Point::new(x0, y), Point::new(x1, y)],
                epaisseur: taille * EP,
            });
            if matches!(n, Noeud::Vecteur(_)) {
                let p = taille * 0.14;
                b.morceaux.push(Morceau::Trace {
                    pts: vec![
                        Point::new(x1 - p, y - p * 0.7),
                        Point::new(x1, y),
                        Point::new(x1 - p, y + p * 0.7),
                    ],
                    epaisseur: taille * EP,
                });
            }
            b.haut = -y + taille * 0.12;
            b
        }
    }
}

/// Met en page une formule (le contenu entre `$…$`).
pub fn mettre_en_page(src: &str, taille: f32) -> Boite {
    horizontal(&analyser(src), taille)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn textes(b: &Boite) -> Vec<String> {
        b.morceaux
            .iter()
            .filter_map(|m| match m {
                Morceau::Texte { texte, .. } => Some(texte.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn fraction_au_dessus_et_au_dessous() {
        let b = mettre_en_page("\\frac{a}{b}", 20.0);
        let ys: Vec<f32> = b
            .morceaux
            .iter()
            .filter_map(|m| match m {
                Morceau::Texte { y, .. } => Some(*y),
                _ => None,
            })
            .collect();
        assert_eq!(textes(&b), ["a", "b"]);
        assert!(ys[0] < ys[1], "le numérateur est au-dessus");
        assert!(
            b.morceaux.iter().any(|m| matches!(m, Morceau::Trace { .. })),
            "une barre"
        );
    }

    #[test]
    fn symboles_et_exposants() {
        let b = mettre_en_page("x^2 + \\alpha \\leq \\sqrt{3} \\in \\mathbb{R}", 20.0);
        let t = textes(&b).concat();
        assert!(t.contains('α') && t.contains('≤') && t.contains('ℝ') && t.contains('2'));
        let b2 = mettre_en_page("x^{n+1}_i", 20.0);
        assert_eq!(
            textes(&b2).concat(),
            "xi n+1".replace(' ', "").replace("n+1", "n+1")
        );
    }

    #[test]
    fn inconnu_ecrit_tel_quel() {
        assert_eq!(textes(&mettre_en_page("\\truc", 20.0)), ["\\truc"]);
    }
}
