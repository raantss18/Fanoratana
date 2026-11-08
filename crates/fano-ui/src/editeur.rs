//! L'éditeur au clavier : taper un texte, corriger une lecture, le mettre en forme ; remplir une case
//! de tableau ; écrire les définitions d'une courbe.

use egui::{Color32, Frame, Margin, RichText, vec2};
use fano_core::{Alignement, Element, Encadre, Liste, Texte};

use crate::app::Fanoratana;
use crate::teinte;
use crate::textes::{T, t};

/// Ce que l'éditeur modifie.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cible {
    /// Un texte (`None` : nouveau).
    Texte(Option<usize>),
    /// Une case de tableau : élément, ligne, colonne.
    Case(usize, usize, usize),
    /// Les définitions d'une courbe (`None` : nouvelle).
    Graphe(Option<usize>),
}

pub struct Editeur {
    pub page: usize,
    pub cible: Cible,
    pub texte: Texte,
    /// Le texte tel qu'il a été lu (pour montrer ce qui a été compris).
    pub lu: String,
    pub focus: bool,
}

impl Editeur {
    pub fn nouveau(page: usize, texte: Texte) -> Editeur {
        Editeur {
            page,
            cible: Cible::Texte(None),
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
            cible: Cible::Texte(Some(index)),
            lu,
            texte,
            focus: true,
        }
    }
    pub fn case(page: usize, index: usize, l: usize, c: usize, contenu: String) -> Editeur {
        Editeur {
            page,
            cible: Cible::Case(index, l, c),
            lu: String::new(),
            texte: Texte {
                contenu,
                taille: 22.0,
                ..Default::default()
            },
            focus: true,
        }
    }
    pub fn graphe(page: usize, index: Option<usize>, definitions: &[String]) -> Editeur {
        Editeur {
            page,
            cible: Cible::Graphe(index),
            lu: String::new(),
            texte: Texte {
                contenu: definitions.join("\n"),
                taille: 22.0,
                ..Default::default()
            },
            focus: true,
        }
    }
}

enum Fin {
    Garder,
    Valider,
    Supprimer,
    Encre,
    Tracer,
}

/// Les lignes du texte qui sont des définitions GeoGebra (`f(x)=…`, `A=(1;2)`).
pub fn definitions(contenu: &str) -> Vec<String> {
    contenu
        .split('\n')
        .map(|l| l.trim().trim_matches('$').to_owned())
        .filter(|l| fano_render::expr::definition(l).is_some())
        .collect()
}

impl Fanoratana {
    pub(crate) fn fenetre_editeur(&mut self, ctx: &egui::Context) {
        let Some(mut ed) = self.editeur.take() else { return };
        let l = self.langue;
        let mut fin = Fin::Garder;
        let largeur = (ctx.content_rect().width() - 32.0).min(640.0);
        egui::Area::new("editeur".into())
            .anchor(egui::Align2::CENTER_TOP, vec2(0.0, 16.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                Frame::window(ui.style())
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        ui.set_width(largeur);
                        if !ed.lu.is_empty() {
                            ui.label(RichText::new(t(l, T::Lu)).small().color(teinte::DOUX));
                        }
                        let champ = egui::TextEdit::multiline(&mut ed.texte.contenu)
                            .font(egui::FontId::new(
                                26.0,
                                egui::FontFamily::Name(crate::peinture::MANUSCRIT.into()),
                            ))
                            .text_color(crate::peinture::c32(ed.texte.couleur))
                            .desired_rows(if matches!(ed.cible, Cible::Graphe(_)) {
                                3
                            } else {
                                2
                            })
                            .desired_width(f32::INFINITY)
                            .lock_focus(true);
                        let rep = ui.add(champ);
                        if ed.focus {
                            rep.request_focus();
                            self.plateforme.clavier(true);
                            ed.focus = false;
                        }
                        let aide = if matches!(ed.cible, Cible::Graphe(_)) {
                            T::Definitions
                        } else {
                            T::AideFormule
                        };
                        ui.label(RichText::new(t(l, aide)).small().color(teinte::DOUX));
                        if matches!(ed.cible, Cible::Texte(_)) {
                            self.mise_en_forme(ui, &mut ed);
                        }
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            let valider =
                                egui::Button::new(RichText::new(t(l, T::Valider)).color(Color32::WHITE))
                                    .fill(teinte::GRAPHITE);
                            if ui.add(valider).clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command)
                            {
                                fin = Fin::Valider;
                            }
                            if matches!(ed.cible, Cible::Texte(_))
                                && !definitions(&ed.texte.contenu).is_empty()
                                && ui.button(t(l, T::Tracer)).clicked()
                            {
                                fin = Fin::Tracer;
                            }
                            if !ed.texte.source.is_empty() && ui.button(t(l, T::RevenirEncre)).clicked() {
                                fin = Fin::Encre;
                            }
                            if matches!(ed.cible, Cible::Texte(Some(_)) | Cible::Graphe(Some(_)))
                                && ui
                                    .button(RichText::new(t(l, T::Supprimer)).color(teinte::DANGER))
                                    .clicked()
                            {
                                fin = Fin::Supprimer;
                            }
                            if ui.button(t(l, T::Fermer)).clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
                                fin = match ed.cible {
                                    Cible::Texte(None) | Cible::Graphe(None) => Fin::Supprimer,
                                    _ => Fin::Valider,
                                };
                            }
                        });
                    });
            });
        if matches!(fin, Fin::Garder) {
            self.editeur = Some(ed);
            return;
        }
        self.memoriser();
        let page = ed.page;
        let vide = ed.texte.contenu.trim().is_empty();
        match (fin, ed.cible) {
            (Fin::Garder, _) => {}
            (Fin::Tracer, _) => {
                let defs = definitions(&ed.texte.contenu);
                self.valider_texte(page, ed.cible, ed.texte, vide);
                self.inserer_graphe(defs);
            }
            (Fin::Valider, Cible::Texte(_)) => self.valider_texte(page, ed.cible, ed.texte, vide),
            (Fin::Valider, Cible::Case(i, li, c)) => {
                if let Some(Element::Tableau(tb)) = self
                    .pages()
                    .and_then(|v| v.get_mut(page))
                    .and_then(|p| p.elements.get_mut(i))
                {
                    if let Some(case) = tb.cellules.get_mut(li).and_then(|ligne| ligne.get_mut(c)) {
                        *case = ed.texte.contenu.trim().to_owned();
                    }
                }
            }
            (Fin::Valider, Cible::Graphe(index)) => {
                let defs: Vec<String> = ed
                    .texte
                    .contenu
                    .lines()
                    .map(|x| x.trim().to_owned())
                    .filter(|x| !x.is_empty())
                    .collect();
                match index {
                    Some(i) => {
                        if let Some(Element::Graphe(g)) = self
                            .pages()
                            .and_then(|v| v.get_mut(page))
                            .and_then(|p| p.elements.get_mut(i))
                        {
                            let (x0, x1, y0, y1) = fano_render::scene::bornes_auto(&defs);
                            (g.xmin, g.xmax, g.ymin, g.ymax) = (x0, x1, y0, y1);
                            g.definitions = defs;
                        }
                    }
                    None if !defs.is_empty() => self.inserer_graphe(defs),
                    None => {}
                }
            }
            (Fin::Supprimer, Cible::Texte(Some(i)) | Cible::Graphe(Some(i))) => {
                if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
                    if i < p.elements.len() {
                        p.elements.remove(i);
                    }
                }
            }
            (Fin::Supprimer, _) => {}
            (Fin::Encre, Cible::Texte(Some(i))) => {
                if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
                    if i < p.elements.len() {
                        p.elements.remove(i);
                        for (k, tr) in ed.texte.source.into_iter().enumerate() {
                            p.elements.insert(i + k, Element::Encre(tr));
                        }
                    }
                }
            }
            (Fin::Encre, _) => {}
        }
        self.plateforme.clavier(false);
        self.modifie();
    }

    fn valider_texte(&mut self, page: usize, cible: Cible, texte: Texte, vide: bool) {
        if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
            match cible {
                Cible::Texte(Some(i)) if i < p.elements.len() => {
                    if vide {
                        p.elements.remove(i);
                    } else {
                        p.elements[i] = Element::Texte(texte);
                    }
                }
                _ if !vide => p.elements.push(Element::Texte(texte)),
                _ => {}
            }
        }
    }

    /// Les outils de traitement de texte, rangés en listes et interrupteurs (pas une barre chargée).
    fn mise_en_forme(&mut self, ui: &mut egui::Ui, ed: &mut Editeur) {
        let l = self.langue;
        let tx = &mut ed.texte;
        if !tx.alternatives.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(t(l, T::Autres)).color(teinte::DOUX));
                for a in tx.alternatives.clone() {
                    if ui.button(&a).clicked() {
                        let ancien = std::mem::replace(&mut tx.contenu, a.clone());
                        tx.alternatives.retain(|x| *x != a);
                        tx.alternatives.insert(0, ancien);
                    }
                }
            });
        }
        ui.horizontal_wrapped(|ui| {
            let style = match tx.titre {
                0 => t(l, T::Texte).to_owned(),
                n => format!("{} {n}", t(l, T::Titre)),
            };
            egui::ComboBox::from_id_salt("style")
                .selected_text(style)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut tx.titre, 0, t(l, T::Texte));
                    for n in 1..=3u8 {
                        ui.selectable_value(&mut tx.titre, n, format!("{} {n}", t(l, T::Titre)));
                    }
                });
            let nom_encadre = |e: Encadre| match e {
                Encadre::Aucun => T::AucunEncadre,
                Encadre::Definition => T::Definition,
                Encadre::ARetenir => T::ARetenir,
                Encadre::Exemple => T::Exemple,
                Encadre::Exercice => T::Exercice,
                Encadre::Citation => T::Citation,
            };
            egui::ComboBox::from_id_salt("encadre")
                .selected_text(format!(
                    "{} : {}",
                    t(l, T::Encadre),
                    t(l, nom_encadre(tx.encadre))
                ))
                .show_ui(ui, |ui| {
                    for e in Encadre::TOUS {
                        ui.selectable_value(&mut tx.encadre, e, t(l, nom_encadre(e)));
                    }
                });
            let nom_liste = |x: Liste| match x {
                Liste::Aucune => T::AucuneListe,
                Liste::Puces => T::Puces,
                Liste::Numeros => T::Numeros,
            };
            egui::ComboBox::from_id_salt("liste")
                .selected_text(t(l, nom_liste(tx.liste)))
                .show_ui(ui, |ui| {
                    for x in [Liste::Aucune, Liste::Puces, Liste::Numeros] {
                        ui.selectable_value(&mut tx.liste, x, t(l, nom_liste(x)));
                    }
                });
            let nom_align = |a: Alignement| match a {
                Alignement::Gauche => T::AGauche,
                Alignement::Centre => T::AuCentre,
                Alignement::Droite => T::ADroite,
            };
            egui::ComboBox::from_id_salt("align")
                .selected_text(t(l, nom_align(tx.alignement)))
                .show_ui(ui, |ui| {
                    for a in [Alignement::Gauche, Alignement::Centre, Alignement::Droite] {
                        ui.selectable_value(&mut tx.alignement, a, t(l, nom_align(a)));
                    }
                });
        });
        ui.columns(2, |cols| {
            crate::widgets::interrupteur(&mut cols[0], &mut tx.souligne, t(l, T::Souligne));
            crate::widgets::interrupteur(&mut cols[1], &mut tx.surligne, t(l, T::Surligne));
        });
        ui.horizontal(|ui| {
            for c in crate::app::PALETTE {
                let (r, rep) = ui.allocate_exact_size(vec2(36.0, 36.0), egui::Sense::click());
                ui.painter()
                    .circle_filled(r.center(), 12.0, crate::peinture::c32(c));
                if c == tx.couleur {
                    ui.painter()
                        .circle_stroke(r.center(), 16.0, egui::Stroke::new(2.0, teinte::GRAPHITE));
                }
                if rep.clicked() {
                    tx.couleur = c;
                }
            }
            ui.add(egui::Slider::new(&mut tx.taille, 12.0..=72.0).show_value(false));
        });
    }
}
