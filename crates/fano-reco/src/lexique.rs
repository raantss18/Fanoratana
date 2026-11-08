//! Correction des mots par un lexique français (fréquences FrequencyWords, CC BY-SA 4.0).
//!
//! Le modèle lit les lettres une à une ; il ne sait pas que « voux » n'est pas un mot. On corrige un
//! mot seulement si sa lecture est **incertaine** (confiance moyenne de ses lettres sous un seuil),
//! vers le mot du lexique qui minimise : coût d'édition pondéré − α·log10(fréquence). Les
//! confusions visuelles de l'écriture manuscrite (r/n, o/e, f/j, u/n, i/l…) coûtent moitié moins.
//! Un mot sûr, un nombre, une formule ne sont jamais touchés : un terme rare et juste reste.

use std::collections::HashMap;
use std::sync::OnceLock;

static LEXIQUE: &str = include_str!("../../../assets/lexique/fr.txt");

/// Poids de la fréquence face au coût d'édition (réglés sur les lectures relevées sur le Pad 7).
const ALPHA: f32 = 0.45;
const COUT: f32 = 2.0;
/// Au-dessus, la lecture du mot est sûre : on n'y touche pas.
pub const CONFIANCE_SURE: f32 = 0.93;

struct Lexique {
    freq: HashMap<String, u32>,
    par_longueur: Vec<Vec<(String, f32)>>,
}

fn lexique() -> &'static Lexique {
    static L: OnceLock<Lexique> = OnceLock::new();
    L.get_or_init(|| {
        let mut freq = HashMap::new();
        let mut par_longueur: Vec<Vec<(String, f32)>> = vec![Vec::new(); 27];
        for l in LEXIQUE.lines().filter(|l| !l.starts_with('#')) {
            let Some((mot, c)) = l.split_once(' ') else {
                continue;
            };
            let Ok(c) = c.parse::<u32>() else { continue };
            let n = mot.chars().count();
            if n < par_longueur.len() {
                par_longueur[n].push((mot.to_owned(), (c as f32).log10()));
            }
            freq.insert(mot.to_owned(), c);
        }
        Lexique { freq, par_longueur }
    })
}

/// Lettres que l'écriture manuscrite confond souvent (dans les deux sens).
fn confusables(a: char, b: char) -> bool {
    const PAIRES: [(char, char); 21] = [
        ('r', 'n'),
        ('o', 'e'),
        ('o', 'a'),
        ('a', 'e'),
        ('f', 'j'),
        ('u', 'n'),
        ('u', 'v'),
        ('i', 'l'),
        ('i', 'r'),
        ('c', 'e'),
        ('n', 'm'),
        ('h', 'b'),
        ('t', 'f'),
        ('g', 'q'),
        ('e', 'é'),
        ('e', 'è'),
        ('e', 'ê'),
        ('a', 'à'),
        ('u', 'ù'),
        ('i', 'î'),
        ('c', 'ç'),
    ];
    PAIRES
        .iter()
        .any(|&(x, y)| (x == a && y == b) || (x == b && y == a))
}

/// Distance d'édition pondérée (substitution confusable 0,5 ; insertion d'un i ou d'un accent 0,6).
fn distance(a: &[char], b: &[char], max: f32) -> f32 {
    let (n, m) = (a.len(), b.len());
    let mut prec: Vec<f32> = (0..=m).map(|j| j as f32).collect();
    let mut cour = vec![0.0; m + 1];
    for i in 1..=n {
        cour[0] = i as f32;
        let mut mini = cour[0];
        for j in 1..=m {
            let sub = if a[i - 1] == b[j - 1] {
                0.0
            } else if confusables(a[i - 1], b[j - 1]) {
                0.5
            } else {
                1.0
            };
            let ins = if b[j - 1] == 'i' { 0.6 } else { 1.0 };
            cour[j] = (prec[j - 1] + sub).min(prec[j] + 1.0).min(cour[j - 1] + ins);
            mini = mini.min(cour[j]);
        }
        if mini > max {
            return f32::INFINITY;
        }
        std::mem::swap(&mut prec, &mut cour);
    }
    prec[m]
}

/// Le meilleur mot du lexique pour `mot` (en minuscules), s'il y en a un de proche.
pub fn corriger_mot(mot: &str) -> Option<String> {
    let l = lexique();
    let bas: Vec<char> = mot.to_lowercase().chars().collect();
    let n = bas.len();
    if n < 2 {
        return None;
    }
    // Une seule faute au plus : un mot rare et juste (« médiatrice », absent du lexique) n'est jamais
    // transformé en un mot courant voisin (« pédiatrie »).
    let max = 1.0;
    let actuel = l
        .freq
        .get(&bas.iter().collect::<String>())
        .map_or(f32::NEG_INFINITY, |c| ALPHA * (*c as f32).log10());
    let mut meilleur: Option<(f32, &str)> = None;
    for len in n.saturating_sub(2)..=(n + 2).min(l.par_longueur.len() - 1) {
        for (cand, lf) in &l.par_longueur[len] {
            let c: Vec<char> = cand.chars().collect();
            let d = distance(&bas, &c, max);
            if d > max {
                continue;
            }
            let score = ALPHA * lf - COUT * d;
            if meilleur.is_none_or(|(s, _)| score > s) {
                meilleur = Some((score, cand));
            }
        }
    }
    let (score, cand) = meilleur?;
    (score > actuel && cand.chars().collect::<Vec<_>>() != bas).then(|| cand.to_owned())
}

/// Corrige une ligne lue : `confiances[i]` est la confiance du i-ième caractère de `texte`.
/// Rend la ligne corrigée (la casse de la première lettre de chaque mot est gardée).
pub fn corriger(texte: &str, confiances: &[f32]) -> String {
    let car: Vec<char> = texte.chars().collect();
    let mut res = String::with_capacity(texte.len());
    let mut i = 0;
    while i < car.len() {
        if !car[i].is_alphabetic() {
            res.push(car[i]);
            i += 1;
            continue;
        }
        let debut = i;
        while i < car.len() && (car[i].is_alphabetic() || car[i] == '\'' || car[i] == '-') {
            i += 1;
        }
        let mot: String = car[debut..i].iter().collect();
        let conf = confiances
            .get(debut..i.min(confiances.len()))
            .filter(|c| !c.is_empty())
            .map_or(1.0, |c| c.iter().sum::<f32>() / c.len() as f32);
        let sur = conf >= CONFIANCE_SURE;
        // Un mot entièrement en capitales (sigle) ou collé à des chiffres reste tel quel.
        let sigle = mot.chars().count() > 1 && mot.chars().all(|c| c.is_uppercase());
        let colle =
            car.get(i).is_some_and(|c| c.is_ascii_digit()) || (debut > 0 && car[debut - 1].is_ascii_digit());
        match (!sur && !sigle && !colle).then(|| corriger_mot(&mot)).flatten() {
            Some(c) => {
                let majuscule = mot.chars().next().is_some_and(char::is_uppercase);
                if majuscule {
                    let mut it = c.chars();
                    if let Some(p) = it.next() {
                        res.extend(p.to_uppercase());
                        res.push_str(it.as_str());
                    }
                } else {
                    res.push_str(&c);
                }
            }
            None => res.push_str(&mot),
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    fn incertain(t: &str) -> Vec<f32> {
        vec![0.6; t.chars().count()]
    }

    #[test]
    fn lectures_du_pad7() {
        // Écriture au stylet d'Antsa, Pad 7, 2026-11-08.
        let t = "fe voux vor";
        assert_eq!(corriger(t, &incertain(t)), "je veux voir");
        let t = "1- Impontant";
        assert_eq!(corriger(t, &incertain(t)), "1- Important");
    }

    #[test]
    fn un_mot_sur_ou_rare_et_juste_reste() {
        let t = "la médiatrice";
        assert_eq!(corriger(t, &incertain(t)), "la médiatrice");
        let t = "voux";
        assert_eq!(
            corriger(t, &vec![0.99; 4]),
            "voux",
            "lecture sûre : on n'y touche pas"
        );
        let t = "la pédiatrie";
        assert_eq!(corriger(t, &incertain(t)), "la pédiatrie");
        let t = "ADN 1789 x2";
        assert_eq!(corriger(t, &incertain(t)), "ADN 1789 x2");
    }
}
