//! La lecture Google (ML Kit Digital Ink, comme Gboard ; ADR 0012) : l'accord de la personne, la
//! langue de l'écriture, le modèle (téléchargé une fois, rafraîchi chaque mois en arrière-plan).

use egui::{Color32, RichText};

use crate::app::Fanoratana;
use crate::plateforme::EtatModele;
use crate::teinte;
use crate::textes::{T, t};
use crate::widgets;

const LANGUES: [(&str, &str); 3] = [("fr", "Français"), ("en", "English"), ("mg", "Malagasy")];
const MOIS: u64 = 30 * 24 * 3600;

fn maintenant_s() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

impl Fanoratana {
    /// Une fois par mois, redemander le modèle de la langue (sans le supprimer d'abord : il reste utilisable).
    /// Avant le premier choix, demander l'accord si la lecture Google existe ici.
    pub(crate) fn rafraichir_modele_encre(&mut self) {
        let langue = self.reglages.langue_ecriture.clone();
        match self.reglages.lecture_google {
            None => {
                if !self.demande_google && self.plateforme.encre_etat(&langue).is_some() {
                    self.demande_google = true;
                }
            }
            Some(true) => {
                let etat = self.plateforme.encre_etat(&langue);
                let vieux = maintenant_s().saturating_sub(self.reglages.maj_modele_encre) > MOIS;
                if etat == Some(EtatModele::Absent) || (etat == Some(EtatModele::Pret) && vieux) {
                    self.plateforme.encre_telecharger(&langue);
                    self.reglages.maj_modele_encre = maintenant_s();
                    self.reglages.ecrire(&self.dossier);
                }
            }
            Some(false) => {}
        }
    }

    /// La demande d'accord, une seule fois.
    pub(crate) fn demande_lecture_google(&mut self, ctx: &egui::Context) {
        if !self.demande_google || self.reglages.lecture_google.is_some() {
            return;
        }
        let l = self.langue;
        let mut choix = None;
        egui::Modal::new("accord-google".into()).show(ctx, |ui| {
            ui.set_width(460.0_f32.min(ctx.content_rect().width() - 48.0));
            ui.heading(t(l, T::GoogleTitre));
            ui.add_space(6.0);
            ui.label(t(l, T::GoogleExplication));
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                let oui = egui::Button::new(RichText::new(t(l, T::GoogleActiver)).color(Color32::WHITE))
                    .fill(teinte::GRAPHITE);
                if ui.add(oui).clicked() {
                    choix = Some(true);
                }
                if ui.button(t(l, T::GoogleRefuser)).clicked() {
                    choix = Some(false);
                }
            });
        });
        if let Some(c) = choix {
            self.reglages.lecture_google = Some(c);
            self.demande_google = false;
            if c {
                let langue = self.reglages.langue_ecriture.clone();
                self.plateforme.encre_telecharger(&langue);
                self.reglages.maj_modele_encre = maintenant_s();
            }
            self.reglages.ecrire(&self.dossier);
        }
    }

    /// La section des Réglages : lecture Google, langue de l'écriture, état du modèle.
    pub(crate) fn reglage_lecture_google(&mut self, ui: &mut egui::Ui) {
        let l = self.langue;
        let langue = self.reglages.langue_ecriture.clone();
        let Some(etat) = self.plateforme.encre_etat(&langue) else {
            ui.label(RichText::new(t(l, T::GoogleIndisponible)).color(teinte::DOUX));
            return;
        };
        let mut actif = self.reglages.lecture_google == Some(true);
        if widgets::interrupteur(ui, &mut actif, t(l, T::GoogleTitre)).changed() {
            self.reglages.lecture_google = Some(actif);
            if actif {
                self.plateforme.encre_telecharger(&langue);
            }
        }
        widgets::section(ui, t(l, T::LangueEcriture));
        ui.horizontal(|ui| {
            for (code, nom) in LANGUES {
                if ui
                    .selectable_label(self.reglages.langue_ecriture == code, nom)
                    .clicked()
                    && self.reglages.langue_ecriture != code
                {
                    self.reglages.langue_ecriture = code.to_owned();
                    if self.reglages.lecture_google == Some(true) {
                        self.plateforme.encre_telecharger(code);
                    }
                }
            }
        });
        let texte = match etat {
            EtatModele::Pret => t(l, T::ModelePret),
            EtatModele::Telechargement => t(l, T::ModeleTelechargement),
            EtatModele::Absent => t(l, T::ModeleAbsent),
            EtatModele::Erreur => t(l, T::ModeleErreur),
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new(texte).color(teinte::DOUX));
            if matches!(etat, EtatModele::Absent | EtatModele::Erreur | EtatModele::Pret)
                && ui.button(t(l, T::ModeleMettreAJour)).clicked()
            {
                self.plateforme.encre_telecharger(&langue);
                self.reglages.maj_modele_encre = maintenant_s();
            }
        });
        if etat == EtatModele::Telechargement {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(500));
        }
    }
}
