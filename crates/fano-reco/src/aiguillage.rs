//! Texte ou formule ? (ADR 0014) Une ligne lue est aiguillée sans rien demander :
//!
//! - **formule** si elle porte des signes d'opération (`=`, `+`, `<`, `^`…) et que ses mots ne sont pas du
//!   français (« pa fo= moto » pour `f(x)=mx+p` : « pa », « fo » ne sont pas des mots) ;
//! - **texte** sinon ; dans une phrase, les maths se mettent d'elles-mêmes entre `$` : une lettre seule est
//!   une variable (« Soit $f$ la fonction »), `R`, `N`, `Z`, `Q`, `C` après « sur », « dans », « de », `∈`
//!   sont des ensembles (`$\mathbb{R}$`), une expression collée (`f(x)`, `2x+1`) est une formule.
//!
//! La règle donnée à l'utilisateur prime : ce qu'il entoure de deux `$` écrits à la main est une formule.

use crate::lexique;

/// Signes qui ne s'écrivent que dans une formule.
const SIGNES: &str = "=+<>^≤≥≠×÷√∫∑Σ∞→⇒⇔∈";

/// Mots qui restent des mots même entre deux signes (fonctions, liaisons d'une condition).
const MOTS_DES_MATHS: [&str; 14] = [
    "sin", "cos", "tan", "lim", "exp", "log", "ln", "det", "si", "et", "ou", "sinon", "pour", "max",
];

/// Les mots (lettres seules exclues) d'une lecture.
fn mots(texte: &str) -> Vec<String> {
    texte
        .split(|c: char| !c.is_alphabetic() && c != '\'' && c != '’')
        .filter(|m| m.chars().count() >= 2)
        .map(str::to_lowercase)
        .collect()
}

/// La ligne est-elle une formule ? Des signes d'opération, et une majorité de « mots » qui n'en sont pas.
pub fn est_maths(texte: &str) -> bool {
    let t = texte.trim();
    if t.is_empty() {
        return false;
    }
    let signes = t.chars().filter(|c| SIGNES.contains(*c)).count();
    // `f(x)`, `g(t)` : une lettre et sa variable entre parenthèses.
    let appel = t
        .as_bytes()
        .windows(4)
        .any(|w| w[0].is_ascii_alphabetic() && w[1] == b'(' && w[2].is_ascii_alphanumeric() && w[3] == b')');
    if signes == 0 && !appel {
        return false;
    }
    let m = mots(t);
    let francais = m
        .iter()
        .filter(|w| !MOTS_DES_MATHS.contains(&w.as_str()) && lexique::est_un_mot(w) && w.chars().count() >= 3)
        .count();
    // Une phrase avec un « = » garde la majorité de vrais mots (« les États généraux = Versailles »).
    francais * 2 < m.len().max(1) || (m.is_empty() && (signes > 0 || appel))
}

/// Du LaTeX écrit à la main : une barre oblique inverse suivie d'un nom de commande.
pub fn est_latex_source(texte: &str) -> bool {
    texte.match_indices('\\').any(|(i, _)| {
        texte[i + 1..]
            .chars()
            .take_while(char::is_ascii_alphabetic)
            .count()
            >= 2
    })
}

/// Une lecture en texte brut (Google, PP-OCR) écrite en LaTeX.
pub fn vers_latex(texte: &str) -> String {
    let mut s = String::new();
    for c in texte.trim().chars() {
        match c {
            '×' => s.push_str("\\times "),
            '÷' => s.push_str("\\div "),
            '≤' => s.push_str("\\leq "),
            '≥' => s.push_str("\\geq "),
            '≠' => s.push_str("\\neq "),
            '→' => s.push_str("\\to "),
            '∞' => s.push_str("\\infty "),
            '∈' => s.push_str("\\in "),
            '²' => s.push_str("^{2}"),
            '³' => s.push_str("^{3}"),
            'π' => s.push_str("\\pi "),
            ' ' => {}
            c => s.push(c),
        }
    }
    s
}

/// Une ligne de texte où les maths sont mises entre `$` (lettres seules, ensembles, expressions collées).
pub fn mathematiser(texte: &str) -> String {
    if texte.contains('$') {
        return texte.to_owned();
    }
    // Une commande LaTeX écrite à la main (`\mathbb{R}`, `\sigma`, `\frac{1}{2}`) : la ligne est du LaTeX.
    if est_latex_source(texte) {
        return format!("${}$", texte.trim());
    }
    let jetons: Vec<&str> = texte.split(' ').collect();
    let mut res: Vec<String> = Vec::with_capacity(jetons.len());
    for (i, j) in jetons.iter().enumerate() {
        let avant = if i > 0 {
            jetons[i - 1].to_lowercase()
        } else {
            String::new()
        };
        // La ponctuation finale reste dans le texte.
        let coeur = j.trim_end_matches([',', '.', ';', ':', '!', '?']);
        let fin = &j[coeur.len()..];
        let mut c = coeur.chars();
        let une_lettre = matches!((c.next(), c.next()), (Some(x), None) if x.is_ascii_alphabetic());
        let ensemble = matches!(coeur, "R" | "N" | "Z" | "Q" | "C" | "R*" | "N*" | "R+" | "R-")
            && matches!(avant.as_str(), "sur" | "dans" | "de" | "∈" | "à" | "vers" | "et");
        let expression = coeur.chars().count() >= 2
            && coeur.chars().any(|c| SIGNES.contains(c) || c == '(')
            && coeur.chars().any(char::is_alphanumeric)
            && est_maths(coeur);
        res.push(if ensemble {
            let (l, suite) = coeur.split_at(1);
            format!("$\\mathbb{{{l}}}{suite}${fin}")
        } else if une_lettre
            && !matches!(
                coeur,
                "a" | "y" | "à" | "A" | "l" | "d" | "s" | "n" | "j" | "m" | "t" | "c" | "qu"
            )
        {
            format!("${coeur}${fin}")
        } else if expression {
            format!("${}${fin}", vers_latex(coeur))
        } else {
            (*j).to_owned()
        });
    }
    res.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texte_ou_formule() {
        // La lecture de Google pour `f(x)=mx+p` (Pad 7, 2026-11-09) : une formule, pas du texte.
        assert!(est_maths("pa fo= moto"));
        assert!(est_maths("f(x)=mx+p"));
        assert!(est_maths("x^2+1=5"));
        assert!(est_maths("f(x)"));
        assert!(!est_maths("Soit f la fonction définie sur R"));
        assert!(!est_maths("En 1789, les États généraux = Versailles"));
        assert!(!est_maths("Les fonctions affines"));
        assert!(!est_maths("Exercice 3 :"));
    }

    #[test]
    fn latex_ecrit_a_la_main() {
        // Lu tel quel par Google sur le Pad 7 (2026-11-09).
        assert_eq!(mathematiser("\\mathbb{R}"), "$\\mathbb{R}$");
        assert_eq!(mathematiser("\\sigma"), "$\\sigma$");
        assert!(!est_latex_source("C'est \\ la fin"));
    }

    #[test]
    fn les_maths_d_une_phrase() {
        assert_eq!(
            mathematiser("Soit f la fonction définie sur R"),
            "Soit $f$ la fonction définie sur $\\mathbb{R}$"
        );
        assert_eq!(
            mathematiser("Pour tout x de N, on a f(x)=2x."),
            "Pour tout $x$ de $\\mathbb{N}$, on a $f(x)=2x$."
        );
        // Les mots d'une lettre du français restent des mots.
        assert_eq!(mathematiser("Il y a une solution"), "Il y a une solution");
        assert_eq!(mathematiser("C'est l'image"), "C'est l'image");
    }
}
