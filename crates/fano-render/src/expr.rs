//! Les définitions façon GeoGebra, telles qu'on les écrit dans un cours : `f(x)=x^2-1`, `y=2x+1`,
//! `A=(1;2)`, en syntaxe GeoGebra ou LaTeX (`\frac{1}{x}`, `\sqrt{x}`, `x^{2}`), multiplication
//! implicite comprise (`2x`, `3(x+1)`, `x\sin(x)`).
//!
//! Analyse par descente récursive (précédence : + −, × ÷, signe, puissance à droite), évaluation en f64.

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Nombre(f64),
    X,
    Neg(Box<Expr>),
    Bin(char, Box<Expr>, Box<Expr>),
    Fonction(&'static str, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Definition {
    Fonction { nom: String, expr: Expr },
    Point { nom: String, x: f64, y: f64 },
}

const FONCTIONS: [&str; 10] = [
    "sqrt", "sin", "cos", "tan", "ln", "log", "exp", "abs", "arctan", "racine",
];

/// Ramène le LaTeX et les signes typographiques à la syntaxe GeoGebra.
pub fn normaliser(s: &str) -> String {
    let mut s = s.trim().trim_matches('$').to_owned();
    // \frac{a}{b} → ((a)/(b)), répété pour les fractions imbriquées.
    while let Some(i) = s.find("\\frac") {
        let reste = &s[i + 5..];
        let (Some((a, apres_a)), true) = (groupe(reste), true) else {
            break;
        };
        let Some((b, apres_b)) = groupe(apres_a) else {
            break;
        };
        let fin = s.len() - apres_b.len();
        s = format!("{}(({})/({})){}", &s[..i], a, b, &s[fin..]);
    }
    let remplacements = [
        ("\\left", ""),
        ("\\right", ""),
        ("\\cdot", "*"),
        ("\\times", "*"),
        ("\\div", "/"),
        ("\\sqrt", "sqrt"),
        ("\\sin", "sin"),
        ("\\cos", "cos"),
        ("\\tan", "tan"),
        ("\\ln", "ln"),
        ("\\log", "log"),
        ("\\exp", "exp"),
        ("\\pi", "pi"),
        ("\\,", ""),
        ("\\;", ""),
        ("\\mathrm", ""),
        ("{", "("),
        ("}", ")"),
        ("[", "("),
        ("]", ")"),
        ("−", "-"),
        ("–", "-"),
        ("×", "*"),
        ("·", "*"),
        ("÷", "/"),
        ("²", "^2"),
        ("³", "^3"),
        ("π", "pi"),
        ("√", "sqrt"),
        ("racine", "sqrt"),
    ];
    for (a, b) in remplacements {
        s = s.replace(a, b);
    }
    s.replace(' ', "")
}

/// `{…}` en tête de `s` (espaces permis) : son contenu et la suite.
fn groupe(s: &str) -> Option<(&str, &str)> {
    let s = s.trim_start();
    if !s.starts_with('{') {
        // Un seul signe : \frac12
        let c = s.chars().next()?;
        return Some((&s[..c.len_utf8()], &s[c.len_utf8()..]));
    }
    let mut prof = 0;
    for (i, c) in s.char_indices() {
        match c {
            '{' => prof += 1,
            '}' => {
                prof -= 1;
                if prof == 0 {
                    return Some((&s[1..i], &s[i + 1..]));
                }
            }
            _ => {}
        }
    }
    None
}

struct Lecteur<'a> {
    s: &'a [u8],
    i: usize,
}

impl Lecteur<'_> {
    fn voir(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }
    fn somme(&mut self) -> Option<Expr> {
        let mut g = self.produit()?;
        while let Some(c @ (b'+' | b'-')) = self.voir() {
            self.i += 1;
            let d = self.produit()?;
            g = Expr::Bin(c as char, Box::new(g), Box::new(d));
        }
        Some(g)
    }
    fn produit(&mut self) -> Option<Expr> {
        let mut g = self.signe()?;
        loop {
            match self.voir() {
                Some(c @ (b'*' | b'/')) => {
                    self.i += 1;
                    let d = self.signe()?;
                    g = Expr::Bin(c as char, Box::new(g), Box::new(d));
                }
                // Multiplication implicite : 2x, 3(x+1), x sin(x), (x+1)(x-1).
                Some(c) if c.is_ascii_alphanumeric() || c == b'(' || c == b'.' => {
                    let d = self.puissance()?;
                    g = Expr::Bin('*', Box::new(g), Box::new(d));
                }
                _ => return Some(g),
            }
        }
    }
    fn signe(&mut self) -> Option<Expr> {
        match self.voir() {
            Some(b'-') => {
                self.i += 1;
                Some(Expr::Neg(Box::new(self.signe()?)))
            }
            Some(b'+') => {
                self.i += 1;
                self.signe()
            }
            _ => self.puissance(),
        }
    }
    fn puissance(&mut self) -> Option<Expr> {
        let base = self.atome()?;
        if self.voir() == Some(b'^') {
            self.i += 1;
            let e = self.signe()?;
            return Some(Expr::Bin('^', Box::new(base), Box::new(e)));
        }
        Some(base)
    }
    fn atome(&mut self) -> Option<Expr> {
        let c = self.voir()?;
        if c == b'(' {
            self.i += 1;
            let e = self.somme()?;
            if self.voir() == Some(b')') {
                self.i += 1;
            }
            return Some(e);
        }
        if c.is_ascii_digit() || c == b'.' {
            let debut = self.i;
            while self.voir().is_some_and(|c| c.is_ascii_digit() || c == b'.') {
                self.i += 1;
            }
            return std::str::from_utf8(&self.s[debut..self.i])
                .ok()?
                .parse()
                .ok()
                .map(Expr::Nombre);
        }
        if c.is_ascii_alphabetic() {
            let reste = std::str::from_utf8(&self.s[self.i..]).ok()?;
            for f in FONCTIONS {
                if reste.starts_with(f) {
                    self.i += f.len();
                    let nom = if f == "racine" { "sqrt" } else { f };
                    let arg = self.puissance()?;
                    return Some(Expr::Fonction(nom, Box::new(arg)));
                }
            }
            if reste.starts_with("pi") {
                self.i += 2;
                return Some(Expr::Nombre(std::f64::consts::PI));
            }
            self.i += 1;
            return match c {
                b'x' => Some(Expr::X),
                b'e' => Some(Expr::Nombre(std::f64::consts::E)),
                _ => None,
            };
        }
        None
    }
}

/// Analyse une expression en `x`.
pub fn expression(s: &str) -> Option<Expr> {
    let n = normaliser(s);
    let mut l = Lecteur {
        s: n.as_bytes(),
        i: 0,
    };
    let e = l.somme()?;
    (l.i == n.len()).then_some(e)
}

/// Reconnaît une définition GeoGebra : fonction ou point.
pub fn definition(s: &str) -> Option<Definition> {
    let n = normaliser(s);
    let (gauche, droite) = n.split_once('=')?;
    // Point : A=(1;2), A=(1,2)
    if gauche.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && droite.starts_with('(')
        && droite.ends_with(')')
    {
        let inner = &droite[1..droite.len() - 1];
        let (a, b) = inner.split_once(';').or_else(|| inner.split_once(','))?;
        let x = evaluer(&expression(a)?, 0.0);
        let y = evaluer(&expression(b)?, 0.0);
        return Some(Definition::Point {
            nom: gauche.to_owned(),
            x,
            y,
        });
    }
    // Fonction : f(x)=…, y=…
    let nom = if let Some(n) = gauche.strip_suffix("(x)") {
        (n.len() <= 2
            && n.chars()
                .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_'))
        .then(|| n.to_owned())?
    } else if gauche == "y" {
        "y".to_owned()
    } else {
        return None;
    };
    let expr = expression(droite)?;
    Some(Definition::Fonction { nom, expr })
}

/// Une définition de fonction mal lue sur sa variable (`f(z)=2x+1`, `g(×)=x^2`) : la variable entre
/// parenthèses est forcément celle du membre de droite. Rend le texte corrigé, ou tel quel.
pub fn corriger_definition(texte: &str) -> String {
    let (dollar, s) = match texte.strip_prefix('$').and_then(|t| t.strip_suffix('$')) {
        Some(t) => (true, t),
        None => (false, texte),
    };
    let Some((gauche, droite)) = s.split_once('=') else {
        return texte.to_owned();
    };
    let g = gauche.trim();
    let (Some(ouvre), true) = (g.find('('), g.ends_with(')')) else {
        return texte.to_owned();
    };
    let nom = &g[..ouvre];
    let var = g[ouvre + 1..g.len() - 1].trim();
    let nom_ok = !nom.is_empty() && nom.chars().count() <= 2 && nom.chars().all(|c| c.is_alphabetic());
    let droite_x = droite.contains('x') && !droite.contains(var);
    if nom_ok && var != "x" && var.chars().count() == 1 && droite_x {
        let corrige = format!("{nom}(x)={}", droite.trim_start());
        return if dollar { format!("${corrige}$") } else { corrige };
    }
    texte.to_owned()
}

pub fn evaluer(e: &Expr, x: f64) -> f64 {
    match e {
        Expr::Nombre(n) => *n,
        Expr::X => x,
        Expr::Neg(a) => -evaluer(a, x),
        Expr::Bin(op, a, b) => {
            let (a, b) = (evaluer(a, x), evaluer(b, x));
            match op {
                '+' => a + b,
                '-' => a - b,
                '*' => a * b,
                '/' => a / b,
                _ => a.powf(b),
            }
        }
        Expr::Fonction(f, a) => {
            let a = evaluer(a, x);
            match *f {
                "sqrt" => a.sqrt(),
                "sin" => a.sin(),
                "cos" => a.cos(),
                "tan" => a.tan(),
                "ln" => a.ln(),
                "log" => a.log10(),
                "exp" => a.exp(),
                "abs" => a.abs(),
                "arctan" => a.atan(),
                _ => f64::NAN,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(s: &str, x: f64) -> f64 {
        match definition(s).unwrap_or_else(|| panic!("non reconnu : {s}")) {
            Definition::Fonction { expr, .. } => evaluer(&expr, x),
            d => panic!("{d:?}"),
        }
    }

    #[test]
    fn variable_mal_lue() {
        // Relevé sur le Pad 7 (2026-11-08) : « f(x)=2x+1 » lu « f(z)=2x+1 ».
        assert_eq!(corriger_definition("f(z)=2x+1"), "f(x)=2x+1");
        assert_eq!(corriger_definition("$g(×)=x^{2}$"), "$g(x)=x^{2}$");
        assert_eq!(
            corriger_definition("f(t)=3t+1"),
            "f(t)=3t+1",
            "une autre variable, cohérente, reste"
        );
        assert_eq!(corriger_definition("Bonjour"), "Bonjour");
        assert!(definition(&corriger_definition("f(z)=2x+1")).is_some());
    }

    #[test]
    fn syntaxes_geogebra_et_latex() {
        assert_eq!(f("f(x)=x^2-1", 3.0), 8.0);
        assert_eq!(f("y = 2x + 1", 2.0), 5.0);
        assert_eq!(f("g(x)=3(x+1)", 1.0), 6.0);
        assert_eq!(f("$f(x)=\\frac{1}{x}$", 4.0), 0.25);
        assert_eq!(f("h(x)=\\sqrt{x}", 9.0), 3.0);
        assert_eq!(f("f(x)=x^{2}+\\frac{x}{2}", 2.0), 5.0);
        assert!((f("f(x)=2sin(x)", std::f64::consts::FRAC_PI_2) - 2.0).abs() < 1e-9);
        assert_eq!(f("f(x) = −x² + 4", 1.0), 3.0);
        assert_eq!(f("f(x)=2^x", 3.0), 8.0);
        assert_eq!(f("f(x)=-x^2", 3.0), -9.0);
        assert_eq!(
            definition("A=(1;2)"),
            Some(Definition::Point {
                nom: "A".into(),
                x: 1.0,
                y: 2.0
            })
        );
        assert_eq!(
            definition("B = (-3, 0.5)"),
            Some(Definition::Point {
                nom: "B".into(),
                x: -3.0,
                y: 0.5
            })
        );
        assert_eq!(definition("Bonjour"), None);
        assert_eq!(definition("x=3"), None);
    }
}
