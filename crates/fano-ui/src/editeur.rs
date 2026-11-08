//! L'éditeur au clavier : taper un texte, corriger une lecture, choisir titre et encadré.

use egui::{Color32, Frame, Margin, RichText, vec2};
use fano_core::{Element, Encadre, Texte};

use crate::app::Fanoratana;
use crate::teinte;
use crate::textes::{T, t};

pub struct Editeur {
    pub page: usize,
    /// `None` : un texte nouveau.
    pub index: Option<usize>,
    pub texte: Texte,
    /// Le texte tel qu'il a été lu (pour montrer ce qui a été compris).
    pub lu: String,
    pub focus: bool,
}

impl Editeur {
    pub fn nouveau(page: usize, texte: Texte) -> Editeur {
        Editeur {
            page,
            index: None,
            lu: String::new(),
            texte,
            focus: true,
        }
    }
    pub fn existant(page: usize, index: usize, texte: Texte) -> Editeur {
        let lu = if texte.source.is_empty() {
            String::new()
        } else {
            texte.contenu.clone()
        };
        Editeur {
            page,
            index: Some(index),
            lu,
            texte,
            focus: true,
        }
    }
}

enum Fin {
    Garder,
    Valider,
    Supprimer,
    Encre,
}

impl Fanoratana {
    pub(crate) fn fenetre_editeur(&mut self, ctx: &egui::Context) {
        let Some(mut ed) = self.editeur.take() else { return };
        let mut fin = Fin::Garder;
        let largeur = (ctx.content_rect().width() - 32.0).min(620.0);
        egui::Area::new("editeur".into())
            .anchor(egui::Align2::CENTER_TOP, vec2(0.0, 16.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                Frame::window(ui.style())
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        ui.set_width(largeur);
                        if !ed.lu.is_empty() {
                            ui.label(RichText::new(t(self.langue, T::Lu)).small().color(teinte::DOUX));
                        }
                        let champ = egui::TextEdit::multiline(&mut ed.texte.contenu)
                            .font(egui::FontId::new(
                                26.0,
                                egui::FontFamily::Name(crate::peinture::MANUSCRIT.into()),
                            ))
                            .text_color(crate::peinture::c32(ed.texte.couleur))
                            .desired_rows(2)
                            .desired_width(f32::INFINITY)
                            .lock_focus(true);
                        let rep = ui.add(champ);
                        if ed.focus {
                            rep.request_focus();
                            self.plateforme.clavier(true);
                            ed.focus = false;
                        }
                        ui.label(
                            RichText::new(t(self.langue, T::AideFormule))
                                .small()
                                .color(teinte::DOUX),
                        );
                        if !ed.texte.alternatives.is_empty() {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new(t(self.langue, T::Autres)).color(teinte::DOUX));
                                for a in ed.texte.alternatives.clone() {
                                    if ui.button(&a).clicked() {
                                        let ancien = std::mem::replace(&mut ed.texte.contenu, a.clone());
                                        ed.texte.alternatives.retain(|x| *x != a);
                                        ed.texte.alternatives.insert(0, ancien);
                                    }
                                }
                            });
                        }
                        ui.horizontal_wrapped(|ui| {
                            for (n, nom) in [(0u8, T::Texte), (1, T::Titre)] {
                                let lab = if n == 0 {
                                    t(self.langue, nom).to_owned()
                                } else {
                                    format!("{} 1", t(self.langue, nom))
                                };
                                if ui.selectable_label(ed.texte.titre == n, lab).clicked() {
                                    ed.texte.titre = n;
                                }
                            }
                            for n in [2u8, 3] {
                                if ui
                                    .selectable_label(
                                        ed.texte.titre == n,
                                        format!("{} {n}", t(self.langue, T::Titre)),
                                    )
                                    .clicked()
                                {
                                    ed.texte.titre = n;
                                }
                            }
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(t(self.langue, T::Encadre)).color(teinte::DOUX));
                            for e in Encadre::TOUS {
                                let nom = match e {
                                    Encadre::Aucun => T::AucunEncadre,
                                    Encadre::Definition => T::Definition,
                                    Encadre::ARetenir => T::ARetenir,
                                    Encadre::Exemple => T::Exemple,
                                    Encadre::Exercice => T::Exercice,
                                    Encadre::Citation => T::Citation,
                                };
                                let (_, c) = fano_render::scene::style_encadre(e);
                                let txt = RichText::new(t(self.langue, nom));
                                let txt = if e == Encadre::Aucun {
                                    txt
                                } else {
                                    txt.color(crate::peinture::c32(c))
                                };
                                if ui.selectable_label(ed.texte.encadre == e, txt).clicked() {
                                    ed.texte.encadre = e;
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            for c in crate::app::PALETTE {
                                let (r, rep) = ui.allocate_exact_size(vec2(36.0, 36.0), egui::Sense::click());
                                ui.painter()
                                    .circle_filled(r.center(), 12.0, crate::peinture::c32(c));
                                if c == ed.texte.couleur {
                                    ui.painter().circle_stroke(
                                        r.center(),
                                        16.0,
                                        egui::Stroke::new(2.0, teinte::GRAPHITE),
                                    );
                                }
                                if rep.clicked() {
                                    ed.texte.couleur = c;
                                }
                            }
                            ui.add(egui::Slider::new(&mut ed.texte.taille, 12.0..=72.0).show_value(false));
                        });
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let valider = egui::Button::new(
                                RichText::new(t(self.langue, T::Valider)).color(Color32::WHITE),
                            )
                            .fill(teinte::ENCRE);
                            if ui.add(valider).clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command)
                            {
                                fin = Fin::Valider;
                            }
                            if !ed.texte.source.is_empty()
                                && ui.button(t(self.langue, T::RevenirEncre)).clicked()
                            {
                                fin = Fin::Encre;
                            }
                            if ed.index.is_some() && ui.button(t(self.langue, T::Supprimer)).clicked() {
                                fin = Fin::Supprimer;
                            }
                            if ui.button(t(self.langue, T::Fermer)).clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
                                fin = if ed.index.is_some() {
                                    Fin::Valider
                                } else {
                                    Fin::Supprimer
                                };
                            }
                        });
                    });
            });
        let page = ed.page;
        match fin {
            Fin::Garder => {
                self.editeur = Some(ed);
                return;
            }
            Fin::Valider => {
                let vide = ed.texte.contenu.trim().is_empty();
                self.memoriser();
                if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
                    match ed.index {
                        Some(i) if i < p.elements.len() => {
                            if vide {
                                p.elements.remove(i);
                            } else {
                                p.elements[i] = Element::Texte(ed.texte);
                            }
                        }
                        _ if !vide => p.elements.push(Element::Texte(ed.texte)),
                        _ => {}
                    }
                }
            }
            Fin::Supprimer => {
                if let Some(i) = ed.index {
                    self.memoriser();
                    if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
                        if i < p.elements.len() {
                            p.elements.remove(i);
                        }
                    }
                }
            }
            Fin::Encre => {
                if let Some(i) = ed.index {
                    self.memoriser();
                    if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
                        if i < p.elements.len() {
                            p.elements.remove(i);
                            for (k, t) in ed.texte.source.into_iter().enumerate() {
                                p.elements.insert(i + k, Element::Encre(t));
                            }
                        }
                    }
                }
            }
        }
        self.plateforme.clavier(false);
        self.modifie();
    }
}
