//! Les mises à jour vues de l'interface : recherche sur un fil, bandeau, installation.

use std::sync::{Arc, Mutex};

use egui::{Color32, Frame, Margin, RichText};
use fano_maj::Version;

use crate::app::Fanoratana;
use crate::teinte;
use crate::textes::{Langue, T, t};

#[derive(Clone, Debug, Default)]
pub enum EtatMaj {
    #[default]
    Rien,
    Recherche {
        demandee: bool,
    },
    AJour {
        demandee: bool,
    },
    Dispo(Version),
    Telechargement(f32),
    Prete(std::path::PathBuf),
    Erreur {
        message: String,
        demandee: bool,
    },
}

pub type Partage = Arc<Mutex<EtatMaj>>;

fn maintenant_s() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

impl Fanoratana {
    /// Cherche une mise à jour (sur un fil) ; `demandee` : la personne l'a demandé (on lui répond même si tout est à jour).
    pub(crate) fn chercher_maj(&mut self, ctx: &egui::Context, demandee: bool) {
        if matches!(
            *self.maj.lock().unwrap(),
            EtatMaj::Recherche { .. } | EtatMaj::Telechargement(_)
        ) {
            return;
        }
        *self.maj.lock().unwrap() = EtatMaj::Recherche { demandee };
        self.reglages.derniere_verif = maintenant_s();
        self.reglages.ecrire(&self.dossier);
        let etat = self.maj.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let r = match fano_maj::chercher(env!("CARGO_PKG_VERSION")) {
                Ok(Some(v)) => EtatMaj::Dispo(v),
                Ok(None) => EtatMaj::AJour { demandee },
                Err(e) => EtatMaj::Erreur { message: e, demandee },
            };
            *etat.lock().unwrap() = r;
            ctx.request_repaint();
        });
    }

    /// Une fois par jour, au démarrage, si le réglage le permet.
    pub(crate) fn maj_quotidienne(&mut self, ctx: &egui::Context) {
        if self.reglages.maj_auto && maintenant_s().saturating_sub(self.reglages.derniere_verif) > 24 * 3600 {
            self.chercher_maj(ctx, false);
        }
    }

    fn telecharger_maj(&mut self, ctx: &egui::Context, v: Version) {
        *self.maj.lock().unwrap() = EtatMaj::Telechargement(0.0);
        let etat = self.maj.clone();
        let dossier = self.dossier.join("mises-a-jour");
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let e2 = etat.clone();
            let c2 = ctx.clone();
            let r = fano_maj::telecharger(&v, &dossier, move |p| {
                *e2.lock().unwrap() = EtatMaj::Telechargement(p);
                c2.request_repaint();
            });
            *etat.lock().unwrap() = match r {
                Ok(p) => EtatMaj::Prete(p),
                Err(message) => EtatMaj::Erreur {
                    message,
                    demandee: true,
                },
            };
            ctx.request_repaint();
        });
    }

    /// Le bandeau des mises à jour, en haut de la feuille, seulement quand il y a quelque chose à dire.
    pub(crate) fn bandeau_maj(&mut self, ui: &mut egui::Ui) {
        let etat = self.maj.lock().unwrap().clone();
        let l = self.langue;
        let texte = |c| t(l, c);
        let mut action = None;
        let message: Option<String> = match &etat {
            EtatMaj::Rien | EtatMaj::Recherche { demandee: false } | EtatMaj::AJour { demandee: false } => {
                None
            }
            EtatMaj::Erreur { demandee: false, .. } => None,
            EtatMaj::Recherche { demandee: true } => Some(texte(T::MajRecherche).into()),
            EtatMaj::AJour { demandee: true } => Some(texte(T::AJour).into()),
            EtatMaj::Erreur { message, .. } => Some(format!("{} {message}", texte(T::MajErreur))),
            EtatMaj::Dispo(v) => {
                let notes = if l == Langue::Fr { &v.notes_fr } else { &v.notes_en };
                Some(format!(
                    "Fanoratana {} {}. {}",
                    v.version,
                    texte(T::MajDispo),
                    notes.lines().next().unwrap_or("")
                ))
            }
            EtatMaj::Telechargement(p) => Some(format!("{} {:.0} %", texte(T::Telechargement), p * 100.0)),
            EtatMaj::Prete(_) => None,
        };
        let Some(message) = message else { return };
        Frame::new()
            .fill(teinte::GRAPHITE)
            .inner_margin(Margin::symmetric(16, 10))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(message).color(Color32::WHITE));
                    match &etat {
                        EtatMaj::Dispo(v) => {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(texte(T::Installer)).color(teinte::GRAPHITE),
                                    )
                                    .fill(Color32::WHITE),
                                )
                                .clicked()
                            {
                                action = Some(Some(v.clone()));
                            }
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(texte(T::PlusTard)).color(Color32::WHITE),
                                    )
                                    .fill(teinte::GRAPHITE),
                                )
                                .clicked()
                            {
                                action = Some(None);
                            }
                        }
                        EtatMaj::AJour { .. } | EtatMaj::Erreur { .. } => {
                            if ui
                                .add(
                                    egui::Button::new(RichText::new(texte(T::Fermer)).color(Color32::WHITE))
                                        .fill(teinte::GRAPHITE),
                                )
                                .clicked()
                            {
                                action = Some(None);
                            }
                        }
                        _ => {}
                    }
                });
            });
        match action {
            Some(Some(v)) => self.telecharger_maj(&ui.ctx().clone(), v),
            Some(None) => *self.maj.lock().unwrap() = EtatMaj::Rien,
            None => {}
        }
    }

    /// Une mise à jour téléchargée et vérifiée : l'installer.
    pub(crate) fn installer_maj_prete(&mut self) {
        let chemin = match &*self.maj.lock().unwrap() {
            EtatMaj::Prete(p) => p.clone(),
            _ => return,
        };
        *self.maj.lock().unwrap() = EtatMaj::Rien;
        match self.plateforme.installer_maj(&chemin) {
            Ok(m) => {
                let m = if m.is_empty() {
                    t(self.langue, T::MajLinux).to_owned()
                } else {
                    m
                };
                self.dire(m);
            }
            Err(e) => {
                let m = format!("{} {e}", t(self.langue, T::MajErreur));
                self.dire(m);
            }
        }
    }
}
