//! Les commandes flottantes aux quatre coins de la feuille, d'après Fikajiana (docs/design.md § Écran) :
//! la feuille occupe tout l'écran ; chaque commande porte son nom ; un seul bouton Outils à la couleur
//! du stylo, en bas à droite, sous le pouce.

use egui::{Align2, Color32, Frame, Margin, Pos2, Rect, RichText, Sense, Stroke, vec2};

use crate::app::{Fanoratana, ModeLecture, Outil, PALETTE, icone_outil};
use crate::icones::{self, Icone};
use crate::teinte;
use crate::textes::{Langue, T, t};
use crate::widgets;

/// Gris des boutons flottants (contraste du trait noir : 17:1).
pub const BOUTON: Color32 = Color32::WHITE;
const MARGE: f32 = 12.0;

fn ombre() -> egui::Shadow {
    egui::Shadow {
        offset: [0, 2],
        blur: 10,
        spread: 0,
        color: Color32::from_black_alpha(36),
    }
}

/// Un bouton rond de 52 dp : icône au trait ; plein noir s'il est actif.
pub fn rond(ui: &mut egui::Ui, icone: Icone, nom: &str, actif: bool) -> egui::Response {
    let (r, rep) = ui.allocate_exact_size(vec2(52.0, 52.0), Sense::click());
    rep.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, actif, nom));
    let rep = rep.on_hover_text(nom);
    let p = ui.painter();
    let c = r.center();
    p.add(egui::Shape::from(
        ombre().as_shape(Rect::from_center_size(c, vec2(48.0, 48.0)), 24),
    ));
    let fond = if actif {
        teinte::GRAPHITE
    } else if rep.hovered() {
        teinte::LIGNE
    } else {
        BOUTON
    };
    p.circle_filled(c, 24.0, fond);
    if !actif {
        p.circle_stroke(c, 24.0, Stroke::new(1.0, teinte::FILET));
    }
    let trait_ = if actif { Color32::WHITE } else { teinte::GRAPHITE };
    icones::dessiner(p, Rect::from_center_size(c, vec2(26.0, 26.0)), icone, trait_);
    if rep.has_focus() {
        p.circle_stroke(c, 26.0, Stroke::new(2.0, teinte::GRAPHITE));
    }
    rep
}

/// Une pastille nommée (icône + texte), comme « Taper un calcul » dans Fikajiana.
pub fn pastille(ui: &mut egui::Ui, icone: Icone, texte: &str) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(
        texte.to_owned(),
        egui::FontId::proportional(16.0),
        teinte::GRAPHITE,
    );
    let taille = vec2(galley.size().x + 64.0, 48.0);
    let (r, rep) = ui.allocate_exact_size(taille, Sense::click());
    rep.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, texte));
    let p = ui.painter();
    p.add(egui::Shape::from(ombre().as_shape(r.shrink(2.0), 24)));
    p.rect_filled(r, 24.0, if rep.hovered() { teinte::SURVOL } else { BOUTON });
    p.rect_stroke(r, 24.0, Stroke::new(1.0, teinte::FILET), egui::StrokeKind::Inside);
    icones::dessiner(
        p,
        Rect::from_center_size(Pos2::new(r.left() + 28.0, r.center().y), vec2(22.0, 22.0)),
        icone,
        teinte::GRAPHITE,
    );
    p.galley(
        Pos2::new(r.left() + 46.0, r.center().y - galley.size().y / 2.0),
        galley,
        teinte::GRAPHITE,
    );
    if rep.has_focus() {
        p.rect_stroke(
            r,
            24.0,
            Stroke::new(2.0, teinte::GRAPHITE),
            egui::StrokeKind::Outside,
        );
    }
    rep
}

/// Le bouton Outils : un rond à la couleur du stylo, avec le crayon (ou l'icône de l'outil).
fn bouton_outils(ui: &mut egui::Ui, app: &Fanoratana) -> egui::Response {
    let (r, rep) = ui.allocate_exact_size(vec2(60.0, 60.0), Sense::click());
    let nom = t(app.langue, T::Outils);
    rep.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, app.outils_ouverts, nom));
    let rep = rep.on_hover_text(nom);
    let p = ui.painter();
    let c = r.center();
    p.add(egui::Shape::from(
        ombre().as_shape(Rect::from_center_size(c, vec2(56.0, 56.0)), 28),
    ));
    p.circle_filled(c, 28.0, if rep.hovered() { teinte::SURVOL } else { BOUTON });
    p.circle_stroke(c, 28.0, Stroke::new(1.0, teinte::FILET));
    if app.outils_ouverts {
        let d = 8.0;
        p.line_segment(
            [c + vec2(-d, -d), c + vec2(d, d)],
            Stroke::new(2.6, teinte::GRAPHITE),
        );
        p.line_segment(
            [c + vec2(-d, d), c + vec2(d, -d)],
            Stroke::new(2.6, teinte::GRAPHITE),
        );
    } else {
        let couleur = crate::peinture::c32(if app.surligneur {
            fano_core::Couleur::SURLIGNEUR
        } else {
            app.couleur
        });
        p.circle_filled(c, 17.0, couleur);
        p.circle_stroke(c, 17.0, Stroke::new(1.0, teinte::FILET));
        let clair = app.couleur.0[..3].iter().map(|&v| u32::from(v)).sum::<u32>() > 600 || app.surligneur;
        let trait_ = if clair { teinte::GRAPHITE } else { Color32::WHITE };
        icones::dessiner(
            p,
            Rect::from_center_size(c, vec2(18.0, 18.0)),
            icone_outil(app.outil),
            trait_,
        );
    }
    if rep.has_focus() {
        p.circle_stroke(c, 31.0, Stroke::new(2.0, teinte::GRAPHITE));
    }
    rep
}

impl Fanoratana {
    /// Les quatre coins.
    pub(crate) fn coins(&mut self, ctx: &egui::Context) {
        let l = self.langue;
        // L'écran moins les barres du système (Android dessine l'app dessous, bord à bord).
        let ppp = ctx.pixels_per_point().max(0.1);
        let [g, h, d, b] = self.plateforme.marges().map(|v| v / ppp);
        let tout = ctx.content_rect();
        let ecran = Rect::from_min_max(tout.min + vec2(g, h), tout.max - vec2(d, b));
        let (g, d, b) = (g, d, b);
        let haut = ecran.top() + self.decalage_haut;
        // Haut gauche : Mes cours et le titre de la leçon.
        egui::Area::new("coin-haut-gauche".into())
            .fixed_pos(Pos2::new(ecran.left() + MARGE, haut + MARGE))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if rond(ui, Icone::Coffre, t(l, T::Coffre), self.tiroir).clicked() {
                        self.tiroir = !self.tiroir;
                        self.outils_ouverts = false;
                    }
                    if let Some(o) = &self.ouverte {
                        let titre = format!("{} — {}", o.fil, o.lecon.titre);
                        let largeur_max = (ecran.width() - 330.0).max(120.0);
                        Frame::new()
                            .fill(BOUTON)
                            .corner_radius(20)
                            .stroke(Stroke::new(1.0, teinte::FILET))
                            .shadow(ombre())
                            .inner_margin(Margin::symmetric(14, 8))
                            .show(ui, |ui| {
                                ui.set_max_width(largeur_max);
                                let rep = ui.add(
                                    egui::Label::new(RichText::new(titre).size(15.0).color(teinte::GRAPHITE))
                                        .truncate()
                                        .sense(Sense::click()),
                                );
                                if rep.clicked() {
                                    self.tiroir = !self.tiroir;
                                }
                            });
                    }
                });
            });
        // Haut droit : Insérer et le Menu.
        egui::Area::new("coin-haut-droit".into())
            .anchor(Align2::RIGHT_TOP, vec2(-MARGE - d, haut - tout.top() + MARGE))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let inserer = pastille(ui, Icone::Inserer, t(l, T::Inserer));
                    egui::Popup::menu(&inserer).show(|ui| self.menu_inserer(ui));
                    let menu = rond(ui, Icone::Plus, t(l, T::Menu), false);
                    egui::Popup::menu(&menu).show(|ui| self.menu_principal(ui));
                });
            });
        // Bas gauche : taper au clavier.
        egui::Area::new("coin-bas-gauche".into())
            .anchor(Align2::LEFT_BOTTOM, vec2(MARGE + g, -MARGE - b))
            .show(ctx, |ui| {
                if pastille(ui, Icone::Clavier, t(l, T::TaperTexte)).clicked() {
                    self.outils_ouverts = false;
                    self.nouveau_texte_clavier();
                }
            });
        // Bas droit : Outils tout à droite (sous le pouce), puis rétablir et annuler.
        egui::Area::new("coin-bas-droit".into())
            .anchor(Align2::RIGHT_BOTTOM, vec2(-MARGE - d, -MARGE - b))
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if bouton_outils(ui, self).clicked() {
                        self.outils_ouverts = !self.outils_ouverts;
                        self.tiroir = false;
                    }
                    ui.add_space(4.0);
                    let peut_annuler = self.ouverte.as_ref().is_some_and(|o| !o.annuler.is_empty());
                    let peut_retablir = self.ouverte.as_ref().is_some_and(|o| !o.retablir.is_empty());
                    ui.add_enabled_ui(peut_retablir, |ui| {
                        if rond(ui, Icone::Retablir, t(l, T::Retablir), false).clicked() {
                            self.retablir();
                        }
                    });
                    ui.add_enabled_ui(peut_annuler, |ui| {
                        if rond(ui, Icone::Annuler, t(l, T::Annuler), false).clicked() {
                            self.annuler();
                        }
                    });
                });
            });
        // La carte Outils, au-dessus du bouton.
        if self.outils_ouverts {
            let hauteur = (ecran.height() - 200.0).clamp(240.0, 640.0);
            egui::Area::new("carte-outils".into())
                .anchor(Align2::RIGHT_BOTTOM, vec2(-MARGE - d, -MARGE - 72.0 - b))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    Frame::new()
                        .fill(teinte::PAPIER)
                        .corner_radius(16)
                        .shadow(ombre())
                        .stroke(Stroke::new(1.0, teinte::FILET))
                        .inner_margin(Margin::same(10))
                        .show(ui, |ui| {
                            ui.set_width(320.0);
                            egui::ScrollArea::vertical()
                                .max_height(hauteur)
                                .show(ui, |ui| self.carte_outils(ui));
                        });
                });
        }
        // Mes cours, par-dessus la feuille (il ne la pousse plus).
        if self.tiroir {
            egui::Area::new("mes-cours".into())
                .fixed_pos(Pos2::new(ecran.left() + MARGE, haut + MARGE + 60.0))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    Frame::new()
                        .fill(teinte::PAPIER)
                        .corner_radius(16)
                        .shadow(ombre())
                        .stroke(Stroke::new(1.0, teinte::FILET))
                        .inner_margin(Margin::symmetric(16, 12))
                        .show(ui, |ui| {
                            ui.set_width(340.0_f32.min(ecran.width() - 2.0 * MARGE - 32.0));
                            ui.set_max_height(ecran.height() - 120.0);
                            self.coffre_ui(ui);
                        });
                });
        }
        if self.reglages_ouverts {
            self.ecran_reglages(ctx);
        }
        if self.a_propos_ouvert {
            self.ecran_a_propos(ctx);
        }
        self.demande_lecture_google(ctx);
    }

    /// La carte Outils : ce qu'on change souvent, chaque ligne nommée ; choisir un outil la referme.
    fn carte_outils(&mut self, ui: &mut egui::Ui) {
        let l = self.langue;
        widgets::section(ui, t(l, T::Couleur));
        ui.horizontal_wrapped(|ui| {
            for c in PALETTE {
                let (r, rep) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::click());
                let choisi = c == self.couleur && !self.surligneur;
                ui.painter()
                    .circle_filled(r.center(), 15.0, crate::peinture::c32(c));
                if choisi {
                    ui.painter()
                        .circle_stroke(r.center(), 19.0, Stroke::new(2.5, teinte::GRAPHITE));
                }
                if rep.clicked() {
                    self.couleur = c;
                    self.surligneur = false;
                    if !matches!(self.outil, Outil::Ecrire | Outil::Formes) {
                        self.outil = Outil::Ecrire;
                    }
                }
            }
        });
        ui.add(egui::Slider::new(&mut self.epaisseur, 1.0..=8.0).text(t(l, T::Epaisseur)));
        ui.separator();
        let surligneur = self.surligneur;
        for (o, icone, nom, surl) in [
            (Outil::Ecrire, Icone::Ecrire, T::Stylo, false),
            (Outil::Ecrire, Icone::Surligneur, T::Surligneur, true),
            (Outil::Formes, Icone::Formes, T::Formes, false),
            (Outil::Selection, Icone::Selection, T::Selection, false),
            (Outil::Gomme, Icone::Gomme, T::Gomme, false),
            (Outil::Clavier, Icone::Clavier, T::TexteOuJeTouche, false),
        ] {
            let choisi = self.outil == o && (o != Outil::Ecrire || surligneur == surl);
            if widgets::ligne(ui, Some(icone), t(l, nom), choisi).clicked() {
                self.outil = o;
                self.surligneur = surl;
                self.outils_ouverts = false;
            }
        }
        ui.separator();
        let avant = self.reglages.doigt_ecrit;
        widgets::interrupteur(ui, &mut self.reglages.doigt_ecrit, t(l, T::DoigtEcrit));
        if self.reglages.doigt_ecrit != avant {
            self.reglages.ecrire(&self.dossier);
        }
        widgets::section(ui, t(l, T::LireComme));
        ui.horizontal(|ui| {
            for (m, nom) in [
                (ModeLecture::Auto, T::LireAuto),
                (ModeLecture::Texte, T::Texte),
                (ModeLecture::Formule, T::LireFormule),
            ] {
                ui.selectable_value(&mut self.mode_lecture, m, t(l, nom));
            }
        });
        if widgets::ligne(ui, Some(Icone::AuPropre), t(l, T::AuPropre), false).clicked() {
            self.outils_ouverts = false;
            self.au_propre_force();
        }
        ui.separator();
        // Alternative aux gestes à deux doigts (WCAG 2.2, critère 2.5.7), comme dans Fikajiana.
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new(format!("+ {}", t(l, T::ZoomPlus))))
                .clicked()
            {
                self.zoom = (self.zoom * 1.25).min(5.0);
            }
            if ui
                .button(RichText::new(format!("− {}", t(l, T::ZoomMoins))))
                .clicked()
            {
                self.zoom = (self.zoom / 1.25).max(0.25);
            }
            if ui.button(t(l, T::Recentrer)).clicked() {
                self.recadrer = true;
            }
        });
    }

    /// Le Menu (haut droit) : ce qu'on fait parfois.
    fn menu_principal(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(300.0);
        let l = self.langue;
        if widgets::ligne(ui, Some(Icone::Diapos), t(l, T::Diapos), false).clicked() && self.ouverte.is_some()
        {
            self.presentation = Some(self.page_vue);
            ui.close();
        }
        ui.menu_button(t(l, T::Exporter), |ui| {
            ui.set_min_width(280.0);
            if widgets::ligne(ui, Some(Icone::Exporter), t(l, T::ExporterPdf), false).clicked() {
                self.exporter("pdf");
                ui.close();
            }
            if widgets::ligne(ui, Some(Icone::Image), t(l, T::ExporterPng), false).clicked() {
                self.exporter("png");
                ui.close();
            }
            if widgets::ligne(ui, Some(Icone::Page), t(l, T::ExporterMd), false).clicked() {
                self.exporter("md");
                ui.close();
            }
        });
        ui.menu_button(t(l, T::Fond), |ui| {
            ui.set_min_width(220.0);
            let page = self.page_vue;
            let actuel = self
                .ouverte
                .as_ref()
                .and_then(|o| o.lecon.pages.get(page))
                .map(|p| p.fond);
            for (f, nom) in [
                (fano_core::Fond::Blanc, T::FondBlanc),
                (fano_core::Fond::Lignes, T::FondLignes),
                (fano_core::Fond::Carreaux, T::FondCarreaux),
                (fano_core::Fond::Points, T::FondPoints),
            ] {
                if widgets::ligne(ui, None, t(l, nom), actuel == Some(f)).clicked() {
                    self.memoriser();
                    if let Some(p) = self.pages().and_then(|p| p.get_mut(page)) {
                        p.fond = f;
                    }
                    self.modifie();
                }
            }
        });
        let n = self.ouverte.as_ref().map_or(0, |o| o.lecon.pages.len());
        if n > 1 && widgets::ligne(ui, Some(Icone::Page), t(l, T::SupprimerPage), false).clicked() {
            let page = self.page_vue.min(n - 1);
            self.memoriser();
            if let Some(o) = &mut self.ouverte {
                o.lecon.pages.remove(page);
            }
            self.modifie();
            ui.close();
        }
        ui.separator();
        if widgets::ligne(ui, Some(Icone::Reglages), t(l, T::Reglages), false).clicked() {
            self.reglages_ouverts = true;
            ui.close();
        }
        if widgets::ligne(ui, None, t(l, T::APropos), false).clicked() {
            self.a_propos_ouvert = true;
            ui.close();
        }
    }

    /// À propos : l'app, sa version, sa licence, et comment joindre son autrice.
    fn ecran_a_propos(&mut self, ctx: &egui::Context) {
        let l = self.langue;
        let mut fermer = false;
        let largeur = 440.0_f32.min(ctx.content_rect().width() - 48.0);
        let rep = egui::Modal::new("a-propos".into()).show(ctx, |ui| {
            ui.set_width(largeur);
            ui.set_max_width(largeur);
            ui.horizontal(|ui| {
                ui.heading(format!("Fanoratana {}", env!("CARGO_PKG_VERSION")));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t(l, T::Fermer)).clicked() {
                        fermer = true;
                    }
                });
            });
            ui.add_space(6.0);
            ui.label(t(l, T::AProposTexte));
            ui.separator();
            widgets::section(ui, t(l, T::Autrice));
            ui.label(RichText::new(AUTRICE).strong());
            let lien = |ui: &mut egui::Ui, nom: &str, texte: &str, url: &str| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(format!("{nom} :")).color(teinte::DOUX));
                    ui.hyperlink_to(texte, url);
                });
            };
            lien(ui, t(l, T::Courriel), COURRIEL, &format!("mailto:{COURRIEL}"));
            lien(
                ui,
                t(l, T::Telephone),
                TELEPHONE,
                &format!("tel:{}", TELEPHONE.replace(' ', "")),
            );
            lien(ui, "WhatsApp", TELEPHONE, WHATSAPP);
            lien(
                ui,
                "GitHub",
                "github.com/raantss18",
                "https://github.com/raantss18",
            );
            ui.separator();
            lien(
                ui,
                t(l, T::CodeSource),
                "github.com/raantss18/Fanoratana",
                "https://github.com/raantss18/Fanoratana",
            );
            ui.label(RichText::new(t(l, T::LicenceTexte)).small().color(teinte::DOUX));
        });
        if fermer || rep.should_close() {
            self.a_propos_ouvert = false;
        }
    }

    /// Les Réglages : un écran à part, en sections (comme Fikajiana).
    fn ecran_reglages(&mut self, ctx: &egui::Context) {
        let l = self.langue;
        let mut fermer = false;
        let rep = egui::Modal::new("reglages".into()).show(ctx, |ui| {
            // Jamais plus large que l'écran (fenêtre flottante, portrait) : le texte va à la ligne.
            let largeur = 480.0_f32.min(ctx.content_rect().width() - 48.0);
            ui.set_width(largeur);
            ui.set_max_width(largeur);
            ui.horizontal(|ui| {
                ui.heading(t(l, T::Reglages));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t(l, T::Fermer)).clicked() {
                        fermer = true;
                    }
                });
            });
            egui::ScrollArea::vertical()
                .max_height(ctx.content_rect().height() - 160.0)
                .show(ui, |ui| {
                    let avant = self.reglages.clone();
                    widgets::section(ui, t(l, T::Ecriture));
                    widgets::interrupteur(ui, &mut self.reglages.doigt_ecrit, t(l, T::DoigtEcrit));
                    widgets::interrupteur(ui, &mut self.reglages.au_propre_auto, t(l, T::AuPropreAuto));
                    ui.label(t(l, T::Delai));
                    ui.spacing_mut().slider_width = (ui.available_width() - 120.0).max(80.0);
                    ui.add(egui::Slider::new(&mut self.reglages.delai_ms, 400..=4000).suffix(" ms"));
                    ui.separator();
                    widgets::section(ui, t(l, T::LectureEcriture));
                    self.reglage_lecture_google(ui);
                    self.reglage_modele_formules(ui);
                    ui.label(RichText::new(t(l, T::AstuceDollar)).small().color(teinte::DOUX));
                    ui.separator();
                    widgets::section(ui, t(l, T::Langue));
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(self.langue == Langue::Fr, "Français")
                            .clicked()
                        {
                            self.langue = Langue::Fr;
                            self.reglages.langue = "fr".into();
                        }
                        if ui
                            .selectable_label(self.langue == Langue::En, "English")
                            .clicked()
                        {
                            self.langue = Langue::En;
                            self.reglages.langue = "en".into();
                        }
                    });
                    ui.separator();
                    widgets::section(ui, t(l, T::MisesAJour));
                    widgets::interrupteur(ui, &mut self.reglages.maj_auto, t(l, T::MajAuto));
                    if ui.button(t(l, T::ChercherMaj)).clicked() {
                        self.chercher_maj(&ui.ctx().clone(), true);
                    }
                    ui.separator();
                    widgets::section(ui, t(l, T::Apropos));
                    ui.label(
                        RichText::new(format!(
                            "Fanoratana {} · GPL-3.0-or-later",
                            env!("CARGO_PKG_VERSION")
                        ))
                        .color(teinte::DOUX),
                    );
                    ui.label(
                        RichText::new(self.coffre.racine.display().to_string())
                            .small()
                            .color(teinte::DOUX),
                    );
                    if self.reglages != avant {
                        self.reglages.ecrire(&self.dossier);
                    }
                });
        });
        if fermer || rep.should_close() {
            self.reglages_ouverts = false;
        }
    }
}

/// L'autrice de Fanoratana et comment la joindre (mêmes coordonnées que ses autres apps).
const AUTRICE: &str = "Antsa Malala RANIRIAMANJAKA";
const COURRIEL: &str = "antsa.raniriamanjaka@univ-antananarivo.mg";
const TELEPHONE: &str = "+261 33 43 838 82";
const WHATSAPP: &str = "https://wa.me/261334383882";
