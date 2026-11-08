//! L'écriture d'Antsa dans Fanoratana (Pad 7 au stylet, relevée par le journal de lecture) : les erreurs
//! signalées le 2026-11-09 ne reviennent pas.

use fano_core::Point;

fn traits(nom: &str) -> Vec<Vec<Point>> {
    let chemin = format!("{}/tests/fanoratana/pad7_2026-11-09/{nom}.json", env!("CARGO_MANIFEST_DIR"));
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

/// Ses `$` : un S plus large que haut, coupé d'un trait vertical. Chacun est un `$` complet.
#[test]
fn ses_dollars_sont_reconnus() {
    assert_eq!(fano_ink::dollars(&traits("dollar_sigma")).len(), 1, "le $ devant \\sigma");
    assert_eq!(fano_ink::dollars(&traits("dollar_seul")).len(), 1, "le $ fermant");
    // Ni `\mathbb{R}` ni `f(x)=mx+p` n'ont de `$`.
    assert!(fano_ink::dollars(&traits("mathbb_R")).is_empty());
    assert!(fano_ink::dollars(&traits("f_mx_p")).is_empty());
}
