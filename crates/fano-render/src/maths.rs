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
    /// Un tableau de LaTeX (`cases`, `pmatrix`, `vmatrix`, `bmatrix`, `matrix`, `aligned`) : ses rangées de
    /// cases, ses délimiteurs gauche et droit, et l'alignement (cases à gauche pour `cases`, centrées sinon).
    Tableau {
        rangees: Vec<Vec<Vec<Noeud>>>,
        gauche: Option<char>,
        droite: Option<char>,
        a_gauche: bool,
    },
}

/// Le contenu brut d'un environnement jusqu'à son `\end{nom}` (les environnements imbriqués compris).
fn jusqu_a_la_fin(c: &mut std::iter::Peekable<std::str::Chars<'_>>, nom: &str) -> String {
    let mut brut = String::new();
    let mut prof = 0;
    let fin = format!("\\end{{{nom}}}");
    let debut = format!("\\begin{{{nom}}}");
    while let Some(ch) = c.next() {
        brut.push(ch);
        if brut.ends_with(&debut) {
            prof += 1;
        } else if brut.ends_with(&fin) {
            if prof == 0 {
                brut.truncate(brut.len() - fin.len());
                return brut;
            }
            prof -= 1;
        }
    }
    brut
}

/// Coupe le contenu d'un tableau en rangées (`\\`) et en cases (`&`), hors accolades.
fn couper_tableau(brut: &str) -> Vec<Vec<Vec<Noeud>>> {
    let mut rangees: Vec<Vec<String>> = vec![vec![String::new()]];
    let mut prof = 0;
    let mut c = brut.chars().peekable();
    while let Some(ch) = c.next() {
        match ch {
            '{' => prof += 1,
            '}' => prof -= 1,
            '&' if prof == 0 => {
                rangees.last_mut().unwrap().push(String::new());
                continue;
            }
            '\\' if prof == 0 && c.peek() == Some(&'\\') => {
                c.next();
                rangees.push(vec![String::new()]);
                continue;
            }
            _ => {}
        }
        rangees.last_mut().unwrap().last_mut().unwrap().push(ch);
    }
    rangees
        .into_iter()
        .filter(|r| r.iter().any(|c| !c.trim().is_empty()))
        .map(|r| r.iter().map(|c| analyser(c)).collect())
        .collect()
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
                    "begin" => {
                        let env = self.texte_brut();
                        let brut = jusqu_a_la_fin(&mut self.c, &env);
                        let (gauche, droite) = match env.as_str() {
                            "cases" => (Some('{'), None),
                            "pmatrix" => (Some('('), Some(')')),
                            "bmatrix" => (Some('['), Some(']')),
                            "vmatrix" => (Some('|'), Some('|')),
                            _ => (None, None),
                        };
                        Noeud::Tableau {
                            rangees: couper_tableau(&brut),
                            gauche,
                            droite,
                            a_gauche: matches!(env.as_str(), "cases" | "aligned" | "array"),
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
        Noeud::Tableau {
            rangees,
            gauche,
            droite,
            a_gauche,
        } => tableau(rangees, *gauche, *droite, *a_gauche, taille),
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

/// Un délimiteur dessiné à la hauteur voulue (de `-haut` à `bas`), dans une largeur `l`.
fn delimiteur(c: char, x: f32, haut: f32, bas: f32, l: f32, ep: f32) -> Vec<Morceau> {
    let (y0, y1) = (-haut, bas);
    let ym = (y0 + y1) / 2.0;
    let trace = |pts: Vec<Point>| Morceau::Trace { pts, epaisseur: ep };
    // L'ouvrant est dessiné tel quel ; le fermant est son miroir.
    let (xa, xb, s) = match c {
        '(' | '[' | '{' | '|' => (x + l * 0.8, x + l * 0.2, 1.0),
        _ => (x + l * 0.2, x + l * 0.8, -1.0),
    };
    let _ = s;
    match c {
        '(' | ')' => {
            let pts = (0..=16)
                .map(|i| {
                    let t = i as f32 / 16.0;
                    let a = (t - 0.5) * std::f32::consts::PI;
                    Point::new(xa + (xb - xa) * a.cos(), ym + (y1 - y0) / 2.0 * a.sin())
                })
                .collect();
            vec![trace(pts)]
        }
        '[' | ']' => vec![trace(vec![
            Point::new(xa, y0),
            Point::new(xb, y0),
            Point::new(xb, y1),
            Point::new(xa, y1),
        ])],
        '{' | '}' => {
            let q = (y1 - y0) / 4.0;
            let xm = (xa + xb) / 2.0;
            vec![trace(vec![
                Point::new(xa, y0),
                Point::new(xm, y0 + q * 0.3),
                Point::new(xm, ym - q * 0.3),
                Point::new(xb, ym),
                Point::new(xm, ym + q * 0.3),
                Point::new(xm, y1 - q * 0.3),
                Point::new(xa, y1),
            ])]
        }
        _ => vec![trace(vec![
            Point::new(x + l * 0.5, y0),
            Point::new(x + l * 0.5, y1),
        ])],
    }
}

/// Un tableau (système, matrice) : cases alignées en colonnes, centrées sur l'axe des formules.
fn tableau(
    rangees: &[Vec<Vec<Noeud>>],
    gauche: Option<char>,
    droite: Option<char>,
    a_gauche: bool,
    taille: f32,
) -> Boite {
    let boites: Vec<Vec<Boite>> = rangees
        .iter()
        .map(|r| r.iter().map(|c| horizontal(c, taille)).collect())
        .collect();
    let n_col = boites.iter().map(Vec::len).max().unwrap_or(0);
    let colonnes: Vec<f32> = (0..n_col)
        .map(|j| {
            boites
                .iter()
                .filter_map(|r| r.get(j))
                .map(|b| b.largeur)
                .fold(0.0, f32::max)
        })
        .collect();
    let ecart_col = taille * if a_gauche { 0.6 } else { 0.8 };
    let interligne = taille * 0.25;
    let hauteurs: Vec<(f32, f32)> = boites
        .iter()
        .map(|r| {
            (
                r.iter().map(|b| b.haut).fold(taille * 0.7, f32::max),
                r.iter().map(|b| b.bas).fold(taille * 0.2, f32::max),
            )
        })
        .collect();
    let total: f32 = hauteurs.iter().map(|(h, b)| h + b).sum::<f32>()
        + interligne * hauteurs.len().saturating_sub(1) as f32;
    let axe = taille * 0.3;
    let haut = total / 2.0 + axe;
    let bas = total / 2.0 - axe;
    let ld = taille * 0.4;
    let x_contenu = if gauche.is_some() { ld + taille * 0.1 } else { 0.0 };
    let largeur_contenu: f32 = colonnes.iter().sum::<f32>() + ecart_col * n_col.saturating_sub(1) as f32;
    let mut b = Boite {
        largeur: x_contenu + largeur_contenu + if droite.is_some() { taille * 0.1 + ld } else { 0.0 },
        haut,
        bas,
        morceaux: vec![],
    };
    let mut y = -haut;
    for (r, (h, bs)) in boites.into_iter().zip(&hauteurs) {
        y += h;
        let mut x = x_contenu;
        for (j, c) in r.into_iter().enumerate() {
            let dx = if a_gauche {
                0.0
            } else {
                (colonnes[j] - c.largeur) / 2.0
            };
            b.ajouter(c, x + dx, y);
            x += colonnes[j] + ecart_col;
        }
        y += bs + interligne;
    }
    let ep = taille * EP;
    if let Some(c) = gauche {
        b.morceaux.extend(delimiteur(c, 0.0, haut, bas, ld, ep));
    }
    if let Some(c) = droite {
        b.morceaux
            .extend(delimiteur(c, b.largeur - ld, haut, bas, ld, ep));
    }
    b
}

/// Met en page une formule (le contenu entre `$…$`).
pub fn mettre_en_page(src: &str, taille: f32) -> Boite {
    horizontal(&analyser(src), taille)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systeme_et_matrice() {
        let b = mettre_en_page(
            r"f(x)=\begin{cases} -x & \text{si } x<0 \\ x & \text{si } x\geq 0 \end{cases}",
            20.0,
        );
        let t = textes(&b).concat();
        assert!(
            t.contains("si") && t.contains('≥') && !t.contains("begin") && !t.contains('&'),
            "{t}"
        );
        // Deux rangées l'une sous l'autre : la boîte est plus haute qu'une ligne.
        assert!(b.haut + b.bas > 40.0);
        let m = mettre_en_page(r"\begin{pmatrix}1&8\\-1&7\end{pmatrix}^{-1}", 20.0);
        let t = textes(&m).concat();
        assert!(
            t.contains('1') && t.contains('8') && t.contains('7') && !t.contains("pmatrix"),
            "{t}"
        );
    }

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
