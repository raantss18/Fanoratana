//! Une page entière de cours (la feuille réelle d'Antsa du 8 novembre, Pad 7) : les lignes trouvées et leur lecture.

use fano_core::{Point, Rect};

fn feuille() -> Vec<Vec<Point>> {
    let chemin = format!(
        "{}/tests/fikajiana/pad7/feuille_8nov_0943.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(chemin).unwrap()).unwrap();
    v.as_array()
        .unwrap()
        .iter()
        .map(|t| {
            t["points"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| Point::new(p[0].as_f64().unwrap() as f32, p[1].as_f64().unwrap() as f32))
                .collect()
        })
        .collect()
}

fn boite(t: &[Point]) -> Rect {
    t.iter()
        .fold(Rect::NOTHING, |r, p| r.union(Rect::from_min_size(*p, 0.0, 0.0)))
}

/// La feuille se découpe en ses six expressions (pas en morceaux de fractions), les trois limites se
/// lisent justes depuis la page entière, et la matrice n'est pas inventée (`None`, le recours la lira).
#[test]
fn la_feuille_d_antsa_bloc_par_bloc() {
    let traits = feuille();
    let boites: Vec<Rect> = traits.iter().map(|t| boite(t)).collect();
    let blocs = fano_ink::grouper_blocs(&boites);
    let cl = fano_reco::signes::ClasseurSignes::charger().unwrap();
    let lus: Vec<Option<String>> = blocs
        .iter()
        .map(|l| {
            let ts: Vec<Vec<Point>> = l.traits.iter().map(|&i| traits[i].clone()).collect();
            fano_reco::structure::lire_formule(&cl, &ts).map(|f| f.latex.chars().filter(|c| !c.is_whitespace()).collect())
        })
        .collect();
    for (b, l) in blocs.iter().zip(&lus) {
        eprintln!("bloc y {:6.0}..{:6.0} {:3} traits : {l:?}", b.bbox.min.y, b.bbox.max.y, b.traits.len());
    }
    assert_eq!(blocs.len(), 6);
    assert_eq!(lus[0].as_deref(), Some("3+5="));
    assert_eq!(lus[1].as_deref(), Some(r"\lim_{x\to3}\frac{1}{(x-3)^{2}}="));
    assert_eq!(lus[2].as_deref(), Some(r"\lim_{x\to0}\frac{e^{x}-1}{2x}="));
    assert_eq!(lus[3].as_deref(), Some(r"\lim_{x\to1}\frac{3x-1}{e^{x}-1}="));
    assert_eq!(lus[4], None, "la matrice");
}
