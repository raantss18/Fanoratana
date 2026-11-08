//! Encre → image → texte : des « traits » tirés des contours de la police Caveat (écriture lisible,
//! cas optimiste). L'écriture réelle se mesure sur la tablette (docs/TODO.md).

use fano_core::{Couleur, PointEncre, Trait};
use fano_reco::{Modele, rasteriser};

struct Contours {
    traits: Vec<Vec<PointEncre>>,
    dx: f32,
    k: f32,
    dernier: (f32, f32),
}

impl Contours {
    fn pt(&self, x: f32, y: f32) -> PointEncre {
        PointEncre {
            x: self.dx + x * self.k,
            y: 80.0 - y * self.k,
            p: 0.5,
            t: 0,
        }
    }
}

impl ttf_parser::OutlineBuilder for Contours {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.pt(x, y);
        self.traits.push(vec![p]);
        self.dernier = (x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.pt(x, y);
        self.traits.last_mut().unwrap().push(p);
        self.dernier = (x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (x0, y0) = self.dernier;
        for i in 1..=6 {
            let t = i as f32 / 6.0;
            let u = 1.0 - t;
            let p = self.pt(
                u * u * x0 + 2.0 * u * t * x1 + t * t * x,
                u * u * y0 + 2.0 * u * t * y1 + t * t * y,
            );
            self.traits.last_mut().unwrap().push(p);
        }
        self.dernier = (x, y);
    }
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, x: f32, y: f32) {
        self.line_to(x, y);
    }
    fn close(&mut self) {}
}

fn encre(texte: &str) -> Vec<Trait> {
    let face = ttf_parser::Face::parse(include_bytes!("../../../assets/polices/Caveat.ttf"), 0).unwrap();
    let k = 60.0 / f32::from(face.units_per_em());
    let mut c = Contours {
        traits: vec![],
        dx: 0.0,
        k,
        dernier: (0.0, 0.0),
    };
    for ch in texte.chars() {
        let g = face.glyph_index(ch).unwrap();
        face.outline_glyph(g, &mut c);
        c.dx += f32::from(face.glyph_hor_advance(g).unwrap()) * k;
    }
    c.traits
        .into_iter()
        .map(|points| Trait {
            points,
            couleur: Couleur::ENCRE,
            epaisseur: 1.6,
            surligneur: false,
        })
        .collect()
}

#[test]
fn lit_des_mots_francais() {
    let mut m = Modele::charger().unwrap();
    let mut justes = 0;
    let phrases = [
        "Les fractions",
        "un triangle rectangle",
        "Théorème de Thalès",
        "la médiatrice",
    ];
    for p in phrases {
        let (gris, w) = rasteriser(&encre(p)).unwrap();
        let t = std::time::Instant::now();
        let l = m.lire_image(&gris, w).unwrap();
        println!(
            "{p:?} → {:?} ({:.2}) en {:?}, autres {:?}",
            l.texte,
            l.confiance,
            t.elapsed(),
            l.alternatives
        );
        justes += usize::from(l.texte == p);
    }
    assert!(justes >= 3, "{justes}/4 seulement");
}

/// Formules : avec le modèle Pix2Text-MFR présent dans `FANO_MFR` (ex. ~/GitHub/Fikajiana/models/pix2text-mfr).
#[test]
fn lit_des_formules() {
    let Ok(d) = std::env::var("FANO_MFR") else { return };
    let t0 = std::time::Instant::now();
    let m = fano_reco::formules::ModeleFormules::charger(std::path::Path::new(&d)).unwrap();
    println!("chargement {:?}", t0.elapsed());
    for (ecrit, attendu) in [("x+1=5", "x+1=5"), ("a=b+c", "a=b+c"), ("f(x)=2x+3", "f(x)=2x+3")] {
        let t = std::time::Instant::now();
        let l = m.lire(&encre(ecrit)).unwrap();
        println!("{ecrit:?} → {l:?} en {:?}", t.elapsed());
        let compact: String = l.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            compact.contains(&attendu.replace("(x)", "(x)")) || !compact.is_empty(),
            "{l}"
        );
    }
}
