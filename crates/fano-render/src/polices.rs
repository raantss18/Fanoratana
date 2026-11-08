//! Les polices embarquées et leur mesure.
//!
//! - Caveat (OFL) : la belle écriture manuscrite du « au propre » (ADR 0006).
//! - Noto Sans Math (OFL) : ce que Caveat n'a pas (lettres grecques, ∞, →, ∈, ∑, ℝ…).
//! - Nunito (OFL) : l'interface.

use std::sync::OnceLock;

pub static CAVEAT: &[u8] = include_bytes!("../../../assets/polices/Caveat.ttf");
pub static SYMBOLES: &[u8] = include_bytes!("../../../assets/polices/NotoSansMath-Regular.ttf");
pub static NUNITO: &[u8] = include_bytes!("../../../assets/polices/Nunito.ttf");

/// Les noms de famille, tels que les SVG et l'interface les citent.
pub const FAMILLE_MANUSCRIT: &str = "Caveat";
pub const FAMILLE_SYMBOLES: &str = "Noto Sans Math";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Police {
    Manuscrit,
    Symboles,
}

impl Police {
    pub fn famille(self) -> &'static str {
        match self {
            Police::Manuscrit => FAMILLE_MANUSCRIT,
            Police::Symboles => FAMILLE_SYMBOLES,
        }
    }
}

pub struct Polices {
    manuscrit: ttf_parser::Face<'static>,
    symboles: ttf_parser::Face<'static>,
}

/// Les polices, chargées une fois.
pub fn polices() -> &'static Polices {
    static P: OnceLock<Polices> = OnceLock::new();
    P.get_or_init(|| Polices {
        manuscrit: ttf_parser::Face::parse(CAVEAT, 0).expect("Caveat embarquée"),
        symboles: ttf_parser::Face::parse(SYMBOLES, 0).expect("Noto Sans Math embarquée"),
    })
}

impl Polices {
    fn face(&self, p: Police) -> &ttf_parser::Face<'static> {
        match p {
            Police::Manuscrit => &self.manuscrit,
            Police::Symboles => &self.symboles,
        }
    }

    /// La police qui sait écrire `c` (Caveat d'abord).
    pub fn police_de(&self, c: char) -> Police {
        if c.is_whitespace() || self.manuscrit.glyph_index(c).is_some() {
            Police::Manuscrit
        } else {
            Police::Symboles
        }
    }

    /// Découpe un texte en morceaux d'une seule police.
    pub fn morceaux(&self, texte: &str) -> Vec<(String, Police)> {
        let mut res: Vec<(String, Police)> = Vec::new();
        for c in texte.chars() {
            let p = self.police_de(c);
            match res.last_mut() {
                Some((s, q)) if *q == p => s.push(c),
                _ => res.push((c.to_string(), p)),
            }
        }
        res
    }

    /// Largeur d'un texte d'une seule police, en pt, à la taille `taille`.
    pub fn largeur_police(&self, texte: &str, police: Police, taille: f32) -> f32 {
        let face = self.face(police);
        let upm = f32::from(face.units_per_em());
        let espace = face
            .glyph_index(' ')
            .and_then(|g| face.glyph_hor_advance(g))
            .unwrap_or(250);
        texte
            .chars()
            .map(|c| {
                face.glyph_index(c)
                    .and_then(|g| face.glyph_hor_advance(g))
                    .unwrap_or(espace)
            })
            .map(f32::from)
            .sum::<f32>()
            * taille
            / upm
    }

    /// Largeur d'un texte quelconque (polices mêlées).
    pub fn largeur(&self, texte: &str, taille: f32) -> f32 {
        self.morceaux(texte)
            .iter()
            .map(|(s, p)| self.largeur_police(s, *p, taille))
            .sum()
    }

    /// Hauteur au-dessus de la ligne de base (ascendante) de la police, en pt.
    pub fn ascendante(&self, police: Police, taille: f32) -> f32 {
        let f = self.face(police);
        f32::from(f.ascender()) * taille / f32::from(f.units_per_em())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repli_sur_les_symboles() {
        let p = polices();
        assert_eq!(p.police_de('a'), Police::Manuscrit);
        assert_eq!(p.police_de('é'), Police::Manuscrit);
        assert_eq!(p.police_de('α'), Police::Symboles);
        assert_eq!(
            p.morceaux("x∈ℝ"),
            vec![("x".into(), Police::Manuscrit), ("∈ℝ".into(), Police::Symboles)]
        );
        assert!(p.largeur("bonjour", 20.0) > p.largeur("bon", 20.0));
    }
}
