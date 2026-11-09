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

/// Faut-il chercher une mise à jour maintenant (sans qu'on l'ait demandé) ?
pub(crate) fn faut_chercher(premiere: bool, auto: bool, maintenant: u64, reussie: u64, tentee: u64) -> bool {
    auto && (premiere
        || (maintenant.saturating_sub(reussie) > 6 * 3600 && maintenant.saturating_sub(tentee) > 30 * 60))
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
        self.maj_tentee = maintenant_s();
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

    /// À chaque ouverture, puis toutes les 6 h tant que l'app reste ouverte (Android la garde en arrière-plan),
    /// si le réglage le permet. Seule une recherche réussie compte : une recherche faite avant la publication
    /// (404) ou sans réseau ne repousse pas la suivante d'un jour (Pad 7, 2026-11-09 : 0.0.2 jamais proposée).
    pub(crate) fn maj_quotidienne(&mut self, ctx: &egui::Context) {
        let etat = self.maj.lock().unwrap().clone();
        if matches!(etat, EtatMaj::AJour { .. } | EtatMaj::Dispo(_))
            && self.reglages.derniere_verif < self.maj_tentee
        {
            self.reglages.derniere_verif = self.maj_tentee;
            self.reglages.ecrire(&self.dossier);
        }
        let premiere = !self.maj_lancee;
        self.maj_lancee = true;
        let libre = matches!(
            etat,
            EtatMaj::Rien | EtatMaj::AJour { demandee: false } | EtatMaj::Erreur { demandee: false, .. }
        );
        if libre
            && faut_chercher(
                premiere,
                self.reglages.maj_auto,
                maintenant_s(),
                self.reglages.derniere_verif,
                self.maj_tentee,
            )
        {
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

#[cfg(test)]
mod tests {
    use super::faut_chercher;

    #[test]
    fn une_recherche_ratee_ne_bloque_pas_la_suivante() {
        let h = 3600;
        let t0 = 1_000_000;
        // À chaque ouverture, même si on a cherché il y a une minute.
        assert!(faut_chercher(true, true, t0, t0 - 60, t0 - 60));
        // Réglage coupé : jamais.
        assert!(!faut_chercher(true, false, t0, 0, 0));
        // App restée ouverte : la version publiée après une recherche ratée (404) est proposée dans l'heure.
        assert!(!faut_chercher(false, true, t0, 0, t0 - 10 * 60));
        assert!(faut_chercher(false, true, t0, 0, t0 - 31 * 60));
        // Après une recherche réussie : pas avant 6 h.
        assert!(!faut_chercher(false, true, t0, t0 - 5 * h, t0 - 5 * h));
        assert!(faut_chercher(false, true, t0, t0 - 7 * h, t0 - 7 * h));
    }
}
