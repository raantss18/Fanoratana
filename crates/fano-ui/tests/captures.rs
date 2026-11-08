//! Captures de l'interface (headless, wgpu) : `FANO_CAPTURES=<dossier> cargo test -p fano-ui --test captures`.
//! Sans la variable, le test vérifie seulement que l'écran se construit sans panique.

use std::path::PathBuf;

use egui_kittest::Harness;
use fano_core::*;
use fano_ui::{Fanoratana, Plateforme};

struct Faux(PathBuf);

impl Plateforme for Faux {
    fn dossier_donnees(&self) -> PathBuf {
        self.0.clone()
    }
    fn demander_image(&mut self) {}
    fn image_choisie(&mut self) -> Option<(String, Vec<u8>)> {
        None
    }
    fn enregistrer(&mut self, _: &str, _: &str, _: Vec<u8>) -> Result<Option<String>, String> {
        Ok(None)
    }
    fn langue(&self) -> String {
        "fr_FR".into()
    }
}

fn texte(x: f32, y: f32, s: &str, taille: f32, titre: u8, encadre: Encadre) -> Element {
    Element::Texte(Texte {
        pos: Point::new(x, y),
        contenu: s.into(),
        taille,
        couleur: Couleur::NOIR,
        largeur: 470.0,
        titre,
        encadre,
        source: vec![],
        alternatives: vec![],
    })
}

/// Une ligne d'encre ondulée (comme une écriture pas encore lue).
fn vague(x: f32, y: f32, l: f32) -> Element {
    let points = (0..120)
        .map(|i| {
            let t = i as f32 / 119.0;
            PointEncre {
                x: x + t * l,
                y: y + (t * 38.0).sin() * 7.0 + (t * 11.0).cos() * 2.0,
                p: 0.5,
                t: i * 8,
            }
        })
        .collect();
    Element::Encre(Trait {
        points,
        couleur: Couleur::NOIR,
        epaisseur: 2.2,
        surligneur: false,
    })
}

fn forme(kind: FormeKind, points: Vec<Point>, etiquette: &str) -> Element {
    Element::Forme(Forme {
        kind,
        points,
        couleur: Couleur::NOIR,
        epaisseur: 1.8,
        rempli: false,
        graine: 5,
        etiquette: etiquette.into(),
        source: vec![],
    })
}

fn preparer(dir: &std::path::Path) {
    let coffre = Coffre::ouvrir(dir.join("Cours")).unwrap();
    let n = coffre.creer_niveau("Seconde").unwrap();
    let c = coffre.creer_chapitre(&n, "Histoire").unwrap();
    let chemin = coffre
        .creer_lecon(&c, "La Révolution française", Format::A4)
        .unwrap();
    let mut l = coffre.lire_lecon(&chemin).unwrap();
    let e = &mut l.pages[0].elements;
    e.push(texte(
        72.0,
        40.0,
        "La Révolution française",
        24.0,
        1,
        Encadre::Aucun,
    ));
    e.push(texte(
        72.0,
        110.0,
        "En 1789, les États généraux se réunissent à Versailles.",
        22.0,
        0,
        Encadre::Aucun,
    ));
    e.push(texte(
        80.0,
        200.0,
        "Une révolution est un changement brutal du régime politique.",
        20.0,
        0,
        Encadre::Definition,
    ));
    e.push(forme(
        FormeKind::Rectangle,
        vec![
            Point::new(90.0, 330.0),
            Point::new(230.0, 330.0),
            Point::new(230.0, 380.0),
            Point::new(90.0, 380.0),
        ],
        "1789",
    ));
    e.push(forme(
        FormeKind::Fleche,
        vec![Point::new(240.0, 355.0), Point::new(340.0, 355.0)],
        "",
    ));
    e.push(forme(
        FormeKind::Rectangle,
        vec![
            Point::new(350.0, 330.0),
            Point::new(500.0, 330.0),
            Point::new(500.0, 380.0),
            Point::new(350.0, 380.0),
        ],
        "1792",
    ));
    e.push(forme(
        FormeKind::Ellipse,
        vec![Point::new(160.0, 470.0), Point::new(48.0, 48.0)],
        "",
    ));
    e.push(forme(
        FormeKind::Triangle,
        vec![
            Point::new(280.0, 520.0),
            Point::new(420.0, 520.0),
            Point::new(300.0, 420.0),
        ],
        "",
    ));
    e.push(vague(80.0, 600.0, 300.0));
    e.push(texte(
        80.0,
        660.0,
        "À retenir : $\\frac{1}{2}$ de la population vit à la campagne.",
        20.0,
        0,
        Encadre::ARetenir,
    ));
    coffre.ecrire_lecon(&chemin, &l).unwrap();
    std::fs::write(
        dir.join("reglages.json"),
        format!("{{\"derniere_lecon\": {:?}}}", chemin.to_string_lossy()),
    )
    .unwrap();
}

fn capture(nom: &str, taille: egui::Vec2, avant: impl Fn(&mut Harness<'_, Option<Fanoratana>>)) {
    let dir = tempfile::tempdir().unwrap();
    preparer(dir.path());
    let donnees = dir.path().to_path_buf();
    let mut faux = Some(Faux(donnees));
    let mut h = Harness::builder().with_size(taille).build_ui_state(
        move |ui, etat: &mut Option<Fanoratana>| {
            if etat.is_none() {
                if let Some(f) = faux.take() {
                    *etat = Some(Fanoratana::new(ui.ctx(), Box::new(f)));
                }
            }
            if let Some(a) = etat.as_mut() {
                a.logic(&ui.ctx().clone());
                a.ui(ui);
            }
        },
        None,
    );
    h.run_steps(6);
    avant(&mut h);
    h.run_steps(4);
    if let Some(d) = std::env::var_os("FANO_CAPTURES") {
        if let Ok(img) = h.render() {
            img.save(PathBuf::from(d).join(format!("{nom}.png"))).unwrap();
        }
    }
}

#[test]
fn ecrans() {
    capture("tablette-paysage", egui::vec2(1280.0, 800.0), |_| {});
    capture("tablette-portrait", egui::vec2(800.0, 1280.0), |_| {});
    capture("coffre", egui::vec2(1280.0, 800.0), |h| {
        use egui_kittest::kittest::Queryable;
        h.get_by_label("Mes cours").click();
    });
}
