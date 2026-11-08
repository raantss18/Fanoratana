//! « Mes cours » : le coffre, niveau → chapitre → leçon, et la recherche.

use egui::{Color32, RichText, vec2};
use fano_core::Format;
use fano_core::coffre::Genre;

use crate::app::{Dialogue, Fanoratana};
use crate::teinte;
use crate::textes::{T, t};

impl Fanoratana {
    pub(crate) fn coffre_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading(t(self.langue, T::Coffre));
        ui.add_space(6.0);
        ui.add(
            egui::TextEdit::singleline(&mut self.recherche)
                .hint_text(t(self.langue, T::Chercher))
                .desired_width(f32::INFINITY),
        );
        ui.add_space(6.0);
        if !self.recherche.trim().is_empty() {
            let res = self.coffre.chercher(&self.recherche);
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (chemin, fil) in res {
                    if ui.link(fil).clicked() {
                        self.ouvrir(chemin);
                        self.tiroir = false;
                    }
                }
            });
            return;
        }
        let ouverte = self.ouverte.as_ref().map(|o| o.chemin.clone());
        let arbre = self.arbre.clone();
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            for niveau in &arbre {
                let id = ui.make_persistent_id(&niveau.chemin);
                let contient = ouverte.as_ref().is_some_and(|o| o.starts_with(&niveau.chemin));
                egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, contient)
                    .show_header(ui, |ui| {
                        let r = ui.label(RichText::new(&niveau.nom).strong().size(18.0));
                        self.menu_entree(
                            &r,
                            &niveau.chemin,
                            &niveau.nom,
                            Some((Genre::Chapitre, T::NouveauChapitre)),
                        );
                    })
                    .body(|ui| {
                        for chap in &niveau.enfants {
                            let id = ui.make_persistent_id(&chap.chemin);
                            let contient = ouverte.as_ref().is_some_and(|o| o.starts_with(&chap.chemin));
                            egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(),
                                id,
                                contient,
                            )
                            .show_header(ui, |ui| {
                                let r = ui.label(RichText::new(&chap.nom).size(17.0));
                                self.menu_entree(
                                    &r,
                                    &chap.chemin,
                                    &chap.nom,
                                    Some((Genre::Lecon, T::NouvelleLecon)),
                                );
                            })
                            .body(|ui| {
                                for lecon in &chap.enfants {
                                    let actif = ouverte.as_ref() == Some(&lecon.chemin);
                                    let r = ui.selectable_label(
                                        actif,
                                        RichText::new(&lecon.nom).color(if actif {
                                            teinte::ENCRE
                                        } else {
                                            teinte::GRAPHITE
                                        }),
                                    );
                                    if r.clicked() {
                                        self.ouvrir(lecon.chemin.clone());
                                        self.tiroir = false;
                                    }
                                    self.menu_entree(&r, &lecon.chemin, &lecon.nom, None);
                                }
                                ui.horizontal(|ui| {
                                    if ui
                                        .small_button(format!("+ {}", t(self.langue, T::NouvelleLecon)))
                                        .clicked()
                                    {
                                        self.dialogue = Some(Dialogue::Nouveau {
                                            genre: Genre::Lecon,
                                            parent: chap.chemin.clone(),
                                            nom: String::new(),
                                            diapo: false,
                                        });
                                    }
                                });
                            });
                        }
                        if ui
                            .small_button(format!("+ {}", t(self.langue, T::NouveauChapitre)))
                            .clicked()
                        {
                            self.dialogue = Some(Dialogue::Nouveau {
                                genre: Genre::Chapitre,
                                parent: niveau.chemin.clone(),
                                nom: String::new(),
                                diapo: false,
                            });
                        }
                    });
            }
            ui.add_space(10.0);
            if ui
                .button(format!("+ {}", t(self.langue, T::NouveauNiveau)))
                .clicked()
            {
                self.dialogue = Some(Dialogue::Nouveau {
                    genre: Genre::Niveau,
                    parent: self.coffre.racine.clone(),
                    nom: String::new(),
                    diapo: false,
                });
            }
            if ui.button(t(self.langue, T::NiveauxType)).clicked() {
                if let Err(e) = self.coffre.creer_niveaux_type() {
                    let m = format!("{} {e}", t(self.langue, T::Erreur));
                    self.dire(m);
                }
                self.rafraichir_arbre();
            }
            ui.add_space(8.0);
            ui.label(
                RichText::new(self.coffre.racine.display().to_string())
                    .small()
                    .color(teinte::DOUX),
            );
        });
    }

    /// Le menu d'une entrée (appui long ou clic droit) : renommer, supprimer, créer dedans.
    fn menu_entree(
        &mut self,
        r: &egui::Response,
        chemin: &std::path::Path,
        nom: &str,
        creer: Option<(Genre, T)>,
    ) {
        r.context_menu(|ui| {
            if let Some((genre, libelle)) = creer {
                if ui.button(t(self.langue, libelle)).clicked() {
                    self.dialogue = Some(Dialogue::Nouveau {
                        genre,
                        parent: chemin.to_path_buf(),
                        nom: String::new(),
                        diapo: false,
                    });
                }
            }
            if ui.button(t(self.langue, T::Renommer)).clicked() {
                self.dialogue = Some(Dialogue::Renommer {
                    chemin: chemin.to_path_buf(),
                    nom: nom.to_owned(),
                });
            }
            if ui
                .button(RichText::new(t(self.langue, T::Supprimer)).color(teinte::DANGER))
                .clicked()
            {
                self.dialogue = Some(Dialogue::Supprimer {
                    chemin: chemin.to_path_buf(),
                    nom: nom.to_owned(),
                });
            }
        });
    }

    pub(crate) fn dialogues(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.dialogue.take() else { return };
        let mut fini = false;
        let mut faire = false;
        let titre = match &d {
            Dialogue::Nouveau {
                genre: Genre::Niveau, ..
            } => t(self.langue, T::NouveauNiveau),
            Dialogue::Nouveau {
                genre: Genre::Chapitre,
                ..
            } => t(self.langue, T::NouveauChapitre),
            Dialogue::Nouveau { .. } => t(self.langue, T::NouvelleLecon),
            Dialogue::Renommer { .. } => t(self.langue, T::Renommer),
            Dialogue::Supprimer { .. } => t(self.langue, T::Supprimer),
        };
        let modal = egui::Modal::new("dialogue".into()).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.heading(titre);
            ui.add_space(8.0);
            match &mut d {
                Dialogue::Nouveau {
                    nom, genre, diapo, ..
                } => {
                    let r = ui.add(
                        egui::TextEdit::singleline(nom)
                            .hint_text(t(self.langue, T::Nom))
                            .desired_width(f32::INFINITY),
                    );
                    r.request_focus();
                    if *genre == Genre::Lecon {
                        ui.horizontal(|ui| {
                            ui.selectable_value(diapo, false, "A4");
                            ui.selectable_value(diapo, true, t(self.langue, T::NouvelleDiapo));
                        });
                    }
                    faire = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                }
                Dialogue::Renommer { nom, .. } => {
                    let r = ui.add(egui::TextEdit::singleline(nom).desired_width(f32::INFINITY));
                    r.request_focus();
                    faire = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                }
                Dialogue::Supprimer { nom, .. } => {
                    ui.label(format!("« {nom} » — {}", t(self.langue, T::MettreCorbeille)));
                }
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let (lib, fond) = match d {
                    Dialogue::Supprimer { .. } => (t(self.langue, T::Supprimer), teinte::DANGER),
                    Dialogue::Nouveau { .. } => (t(self.langue, T::Creer), teinte::ENCRE),
                    Dialogue::Renommer { .. } => (t(self.langue, T::Valider), teinte::ENCRE),
                };
                if ui
                    .add(
                        egui::Button::new(RichText::new(lib).color(Color32::WHITE))
                            .fill(fond)
                            .min_size(vec2(96.0, 40.0)),
                    )
                    .clicked()
                {
                    faire = true;
                }
                if ui.button(t(self.langue, T::Fermer)).clicked() {
                    fini = true;
                }
            });
        });
        if modal.should_close() {
            fini = true;
        }
        if faire {
            let r = match &d {
                Dialogue::Nouveau {
                    genre,
                    parent,
                    nom,
                    diapo,
                } => {
                    let r = match genre {
                        Genre::Niveau => self.coffre.creer_niveau(nom),
                        Genre::Chapitre => self.coffre.creer_chapitre(parent, nom),
                        Genre::Lecon => self.coffre.creer_lecon(
                            parent,
                            nom,
                            if *diapo { Format::Diapo } else { Format::A4 },
                        ),
                    };
                    r.map(|p| {
                        if *genre == Genre::Lecon {
                            self.rafraichir_arbre();
                            self.ouvrir(p);
                            self.tiroir = false;
                        }
                    })
                }
                Dialogue::Renommer { chemin, nom } => self.coffre.renommer(chemin, nom).map(|nouveau| {
                    if let Some(o) = &mut self.ouverte {
                        if o.chemin.starts_with(chemin) {
                            let reste = o
                                .chemin
                                .strip_prefix(chemin)
                                .map(|r| r.to_path_buf())
                                .unwrap_or_default();
                            o.chemin = if reste.as_os_str().is_empty() {
                                nouveau
                            } else {
                                nouveau.join(reste)
                            };
                        }
                    }
                }),
                Dialogue::Supprimer { chemin, .. } => {
                    if self
                        .ouverte
                        .as_ref()
                        .is_some_and(|o| o.chemin.starts_with(chemin))
                    {
                        self.enregistrer();
                        self.ouverte = None;
                    }
                    self.coffre.supprimer(chemin)
                }
            };
            if let Err(e) = r {
                let m = format!("{} {e}", t(self.langue, T::Erreur));
                self.dire(m);
            }
            self.rafraichir_arbre();
            fini = true;
        }
        if !fini {
            self.dialogue = Some(d);
        }
    }
}
