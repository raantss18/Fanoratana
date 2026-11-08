//! L'état de l'app et la mise en place de l'écran.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use egui::{Color32, Frame, Margin, Sense, Stroke, vec2};
use fano_core::{Coffre, Couleur, Element, Entree, Lecon, Page, Texte, Trait};
use fano_reco::Lecteur;

use crate::editeur::Editeur;
use crate::icones::{self, Icone};
use crate::peinture::Images;
use crate::plateforme::Plateforme;
use crate::reglages::Reglages;
use crate::teinte;
use crate::textes::{Langue, T, t};

/// La leçon ouverte.
pub struct Ouverte {
    pub chemin: PathBuf,
    pub lecon: Lecon,
    /// Niveau › Chapitre.
    pub fil: String,
    pub sale: bool,
    pub modifiee: f64,
    pub annuler: Vec<Vec<Page>>,
    pub retablir: Vec<Vec<Page>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outil {
    Ecrire,
    Formes,
    Gomme,
    Selection,
    Clavier,
}

/// Un geste en cours sur la feuille.
#[derive(Clone, Debug)]
pub enum Geste {
    Trait {
        page: usize,
        trait_: Trait,
        debut: f64,
        surligneur: bool,
    },
    Gomme {
        page: usize,
        memorise: bool,
    },
    RectSel {
        page: usize,
        debut: fano_core::Point,
        fin: fano_core::Point,
    },
    Deplacer {
        page: usize,
        dernier: fano_core::Point,
        memorise: bool,
    },
    Defiler,
}

pub enum Dialogue {
    Nouveau {
        genre: fano_core::coffre::Genre,
        parent: PathBuf,
        nom: String,
        diapo: bool,
    },
    Renommer {
        chemin: PathBuf,
        nom: String,
    },
    Supprimer {
        chemin: PathBuf,
        nom: String,
    },
}

/// Une lecture envoyée au modèle : la page, les traits lus, et si on la force.
pub struct EnLecture {
    pub page: usize,
    pub traits: Vec<Trait>,
    pub forcee: bool,
}

pub struct Fanoratana {
    pub(crate) plateforme: Box<dyn Plateforme>,
    pub(crate) dossier: PathBuf,
    pub(crate) reglages: Reglages,
    pub(crate) langue: Langue,
    pub(crate) coffre: Coffre,
    pub(crate) arbre: Vec<Entree>,
    pub(crate) ouverte: Option<Ouverte>,
    pub(crate) outil: Outil,
    pub(crate) couleur: Couleur,
    pub(crate) epaisseur: f32,
    pub(crate) surligneur: bool,
    pub(crate) rempli: bool,
    pub(crate) zoom: f32,
    /// Défilement de la feuille (px écran), et s'il faut recadrer.
    pub(crate) defil: egui::Vec2,
    pub(crate) recadrer: bool,
    pub(crate) geste: Option<Geste>,
    /// L'encre pas encore mise au propre : (page, trait).
    pub(crate) attente: Vec<(usize, Trait)>,
    pub(crate) derniere_ecriture: Option<f64>,
    pub(crate) lecteur: Option<Lecteur>,
    pub(crate) lectures: HashMap<u64, EnLecture>,
    pub(crate) prochain: u64,
    pub(crate) selection: Vec<usize>,
    pub(crate) page_sel: usize,
    pub(crate) page_vue: usize,
    pub(crate) editeur: Option<Editeur>,
    pub(crate) tiroir: bool,
    pub(crate) dialogue: Option<Dialogue>,
    pub(crate) recherche: String,
    pub(crate) presentation: Option<usize>,
    pub(crate) message: Option<(String, f64)>,
    pub(crate) images: Images,
    /// Le dernier segment devenu forme : une pointe dessinée juste après en fait une flèche.
    pub(crate) dernier_segment: Option<(usize, usize, f64)>,
    pub(crate) maintenant: f64,
    /// Le dernier identifiant de toucher vu (pour savoir si c'est un doigt ou un stylet).
    pub(crate) toucher: Option<u64>,
    /// egui ne prend les polices qu'à l'image suivante : la première est sautée.
    polices_pretes: bool,
    pub(crate) maj: crate::maj::Partage,
    maj_lancee: bool,
    pub(crate) largeur_vue: f32,
}

pub const PALETTE: [Couleur; 7] = [
    Couleur::NOIR,
    Couleur::ENCRE,
    Couleur::BLEU,
    Couleur::ROUGE,
    Couleur::VERT,
    Couleur::ORANGE,
    Couleur::VIOLET,
];

impl Fanoratana {
    pub fn new(ctx: &egui::Context, plateforme: Box<dyn Plateforme>) -> Fanoratana {
        crate::installer_polices(ctx);
        crate::appliquer_theme(ctx);
        let dossier = plateforme.dossier_donnees();
        let reglages = Reglages::lire(&dossier);
        let systeme = plateforme.langue();
        let langue = Langue::depuis(if reglages.langue.is_empty() {
            &systeme
        } else {
            &reglages.langue
        });
        let racine = if reglages.coffre.is_empty() {
            dossier.join("Cours")
        } else {
            PathBuf::from(&reglages.coffre)
        };
        let coffre = Coffre::ouvrir(&racine).unwrap_or_else(|e| {
            log::error!("coffre {}: {e}", racine.display());
            Coffre { racine }
        });
        let reveil = ctx.clone();
        let lecteur = Some(Lecteur::demarrer(move || reveil.request_repaint()));
        let mut app = Fanoratana {
            plateforme,
            dossier,
            reglages,
            langue,
            arbre: coffre.arbre(),
            coffre,
            ouverte: None,
            outil: Outil::Ecrire,
            couleur: Couleur::NOIR,
            epaisseur: 2.2,
            surligneur: false,
            rempli: false,
            zoom: 1.0,
            defil: egui::Vec2::ZERO,
            recadrer: true,
            geste: None,
            attente: Vec::new(),
            derniere_ecriture: None,
            lecteur,
            lectures: HashMap::new(),
            prochain: 1,
            selection: Vec::new(),
            page_sel: 0,
            page_vue: 0,
            editeur: None,
            tiroir: false,
            dialogue: None,
            recherche: String::new(),
            presentation: None,
            message: None,
            images: Images::default(),
            dernier_segment: None,
            maintenant: 0.0,
            toucher: None,
            polices_pretes: false,
            maj: Default::default(),
            maj_lancee: false,
            largeur_vue: 0.0,
        };
        // La dernière leçon, ou la première du coffre.
        let derniere = PathBuf::from(&app.reglages.derniere_lecon);
        let a_ouvrir = if derniere.is_file() {
            Some(derniere)
        } else {
            app.arbre
                .iter()
                .flat_map(|n| &n.enfants)
                .flat_map(|c| &c.enfants)
                .map(|l| l.chemin.clone())
                .next()
        };
        if let Some(p) = a_ouvrir {
            app.ouvrir(p);
        }
        app
    }

    pub(crate) fn dire(&mut self, m: impl Into<String>) {
        self.message = Some((m.into(), self.maintenant));
    }

    pub(crate) fn rafraichir_arbre(&mut self) {
        self.arbre = self.coffre.arbre();
    }

    pub(crate) fn ouvrir(&mut self, chemin: PathBuf) {
        self.enregistrer();
        match self.coffre.lire_lecon(&chemin) {
            Ok(lecon) => {
                let fil = chemin
                    .parent()
                    .map(|c| {
                        let n = c.parent().map(fano_core::coffre::nom_montre).unwrap_or_default();
                        format!("{n} › {}", fano_core::coffre::nom_montre(c))
                    })
                    .unwrap_or_default();
                self.reglages.derniere_lecon = chemin.to_string_lossy().into_owned();
                self.reglages.ecrire(&self.dossier);
                self.ouverte = Some(Ouverte {
                    chemin,
                    lecon,
                    fil,
                    sale: false,
                    modifiee: 0.0,
                    annuler: vec![],
                    retablir: vec![],
                });
                self.attente.clear();
                self.lectures.clear();
                self.selection.clear();
                self.editeur = None;
                self.recadrer = true;
                self.page_vue = 0;
            }
            Err(e) => {
                let m = format!("{} {e}", t(self.langue, T::Erreur));
                self.dire(m);
            }
        }
    }

    /// Écrit la leçon si elle a changé.
    pub(crate) fn enregistrer(&mut self) {
        if let Some(o) = &mut self.ouverte {
            if o.sale {
                match self.coffre.ecrire_lecon(&o.chemin, &o.lecon) {
                    Ok(()) => o.sale = false,
                    Err(e) => log::error!("enregistrement : {e}"),
                }
            }
        }
    }

    /// À appeler avant chaque modification : garde l'état pour « Annuler ».
    pub(crate) fn memoriser(&mut self) {
        if let Some(o) = &mut self.ouverte {
            o.annuler.push(o.lecon.pages.clone());
            if o.annuler.len() > 100 {
                o.annuler.remove(0);
            }
            o.retablir.clear();
        }
    }

    pub(crate) fn modifie(&mut self) {
        let now = self.maintenant;
        if let Some(o) = &mut self.ouverte {
            o.sale = true;
            o.modifiee = now;
        }
    }

    pub(crate) fn annuler(&mut self) {
        if let Some(o) = &mut self.ouverte {
            if let Some(p) = o.annuler.pop() {
                o.retablir.push(std::mem::replace(&mut o.lecon.pages, p));
                o.sale = true;
            }
        }
        self.apres_histoire();
    }

    pub(crate) fn retablir(&mut self) {
        if let Some(o) = &mut self.ouverte {
            if let Some(p) = o.retablir.pop() {
                o.annuler.push(std::mem::replace(&mut o.lecon.pages, p));
                o.sale = true;
            }
        }
        self.apres_histoire();
    }

    fn apres_histoire(&mut self) {
        self.selection.clear();
        self.attente.clear();
        self.lectures.clear();
        self.editeur = None;
    }

    pub(crate) fn pages(&mut self) -> Option<&mut Vec<Page>> {
        self.ouverte.as_mut().map(|o| &mut o.lecon.pages)
    }

    /// À la fermeture : tout est écrit.
    pub fn quitter(&mut self) {
        self.enregistrer();
        self.reglages.ecrire(&self.dossier);
    }

    /// Le travail de chaque image, même sans interaction.
    pub fn logic(&mut self, ctx: &egui::Context) {
        self.maintenant = ctx.input(|i| i.time);
        if !self.maj_lancee {
            self.maj_lancee = true;
            self.maj_quotidienne(ctx);
        }
        self.installer_maj_prete();
        self.recevoir_lectures();
        // Mise au propre automatique après la pause.
        if self.reglages.au_propre_auto && self.geste.is_none() && !self.attente.is_empty() {
            if let Some(d) = self.derniere_ecriture {
                let delai = self.reglages.delai_ms as f64 / 1000.0;
                let reste = d + delai - self.maintenant;
                if reste <= 0.0 {
                    let traits: Vec<(usize, Trait)> = std::mem::take(&mut self.attente);
                    self.lire_traits(traits, false);
                } else {
                    ctx.request_repaint_after(Duration::from_secs_f64(reste));
                }
            }
        }
        // Enregistrement automatique, une seconde après le dernier changement.
        if let Some(o) = &self.ouverte {
            if o.sale {
                let reste = o.modifiee + 1.0 - self.maintenant;
                if reste <= 0.0 {
                    self.enregistrer();
                } else {
                    ctx.request_repaint_after(Duration::from_secs_f64(reste));
                }
            }
        }
        if let Some((nom, octets)) = self.plateforme.image_choisie() {
            self.inserer_image(&nom, &octets);
        }
        if !self.lectures.is_empty() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        if let Some((_, quand)) = &self.message {
            let reste = quand + 4.0 - self.maintenant;
            if reste <= 0.0 {
                self.message = None;
            } else {
                ctx.request_repaint_after(Duration::from_secs_f64(reste));
            }
        }
        self.raccourcis(ctx);
    }

    fn raccourcis(&mut self, ctx: &egui::Context) {
        use egui::{Key, Modifiers};
        if ctx.memory(|m| m.focused().is_some()) {
            return;
        }
        let (z, y, shift_z) = ctx.input_mut(|i| {
            (
                i.consume_key(Modifiers::COMMAND, Key::Z),
                i.consume_key(Modifiers::COMMAND, Key::Y),
                i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z),
            )
        });
        if y || shift_z {
            self.retablir();
        } else if z {
            self.annuler();
        }
        if self.presentation.is_some() {
            return;
        }
        let touches = ctx.input_mut(|i| {
            [
                i.consume_key(Modifiers::NONE, Key::F5),
                i.consume_key(Modifiers::NONE, Key::Delete) || i.consume_key(Modifiers::NONE, Key::Backspace),
                i.consume_key(Modifiers::COMMAND, Key::T) || i.consume_key(Modifiers::NONE, Key::Enter),
                i.consume_key(Modifiers::COMMAND, Key::O),
                i.consume_key(Modifiers::NONE, Key::Escape),
            ]
        });
        if touches[0] && self.ouverte.is_some() {
            self.presentation = Some(self.page_vue);
        }
        if touches[1] && !self.selection.is_empty() {
            self.supprimer_selection();
        }
        if touches[2] && self.ouverte.is_some() {
            // Un nouveau texte au clavier, sous ce qui est déjà écrit.
            self.nouveau_texte_clavier();
        }
        if touches[3] {
            self.tiroir = !self.tiroir;
        }
        if touches[4] {
            self.selection.clear();
            self.tiroir = false;
        }
        for (k, o) in [
            (Key::Num1, Outil::Ecrire),
            (Key::Num2, Outil::Formes),
            (Key::Num3, Outil::Gomme),
            (Key::Num4, Outil::Selection),
            (Key::Num5, Outil::Clavier),
        ] {
            if ctx.input_mut(|i| i.consume_key(Modifiers::ALT, k)) {
                self.outil = o;
            }
        }
    }

    /// Ouvre l'éditeur pour un texte neuf, placé sous le dernier élément de la page vue.
    pub(crate) fn nouveau_texte_clavier(&mut self) {
        let page = self.page_vue;
        let Some(o) = &self.ouverte else { return };
        let Some(p) = o.lecon.pages.get(page) else { return };
        let bas = p
            .elements
            .iter()
            .map(|e| fano_render::scene::bbox(e).max.y)
            .fold(48.0_f32, f32::max);
        let pos = fano_core::Point::new(72.0, bas + 16.0);
        self.editeur = Some(Editeur::nouveau(page, self.texte_vide(pos)));
    }

    pub(crate) fn texte_vide(&self, pos: fano_core::Point) -> Texte {
        let largeur = self
            .ouverte
            .as_ref()
            .map_or(440.0, |o| o.lecon.format.taille().0 - pos.x - 40.0)
            .max(120.0);
        Texte {
            pos,
            contenu: String::new(),
            taille: 22.0,
            couleur: self.couleur,
            largeur,
            titre: 0,
            encadre: fano_core::Encadre::Aucun,
            source: vec![],
            alternatives: vec![],
        }
    }

    pub(crate) fn supprimer_selection(&mut self) {
        let page = self.page_sel;
        let mut sel = std::mem::take(&mut self.selection);
        sel.sort_unstable();
        sel.dedup();
        self.memoriser();
        if let Some(p) = self.pages().and_then(|p| p.get_mut(page)) {
            for i in sel.into_iter().rev() {
                if i < p.elements.len() {
                    p.elements.remove(i);
                }
            }
        }
        self.modifie();
    }

    pub(crate) fn inserer_image(&mut self, nom: &str, octets: &[u8]) {
        let Some((w, h)) = crate::peinture::taille_image(octets, nom) else {
            let m = format!("{} {nom}", t(self.langue, T::Erreur));
            self.dire(m);
            return;
        };
        let fichier = match self.coffre.importer_media(nom, octets) {
            Ok(f) => f,
            Err(e) => {
                let m = format!("{} {e}", t(self.langue, T::Erreur));
                self.dire(m);
                return;
            }
        };
        let page = self.page_vue;
        let Some(o) = &self.ouverte else { return };
        let (pw, ph) = o.lecon.format.taille();
        let k = ((pw * 0.6) / w).min((ph * 0.5) / h).min(1.0);
        let (iw, ih) = (w * k, h * k);
        let rect =
            fano_core::Rect::from_min_size(fano_core::Point::new((pw - iw) / 2.0, (ph - ih) / 3.0), iw, ih);
        self.memoriser();
        if let Some(p) = self.pages().and_then(|p| p.get_mut(page)) {
            p.elements
                .push(Element::Image(fano_core::ImageEl { fichier, rect }));
            self.selection = vec![p.elements.len() - 1];
            self.page_sel = page;
            self.outil = Outil::Selection;
        }
        self.modifie();
    }

    /// L'écran.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        if !self.polices_pretes {
            self.polices_pretes = true;
            ctx.request_repaint();
            return;
        }
        if let Some(i) = self.presentation {
            self.presenter(ui, i);
            return;
        }
        egui::Panel::left("rail")
            .exact_size(68.0)
            .resizable(false)
            .frame(Frame::new().fill(teinte::PAPIER))
            .show(ui, |ui| self.rail(ui));
        if self.tiroir {
            egui::Panel::left("coffre")
                .exact_size(320.0)
                .resizable(false)
                .frame(
                    Frame::new()
                        .fill(teinte::PAPIER)
                        .inner_margin(Margin::symmetric(16, 14)),
                )
                .show(ui, |ui| self.coffre_ui(ui));
        }
        egui::CentralPanel::default()
            .frame(Frame::new().fill(teinte::BUREAU))
            .show(ui, |ui| {
                self.bandeau_maj(ui);
                if self.ouverte.is_some() {
                    self.toile(ui);
                } else {
                    ui.centered_and_justified(|ui| ui.label(t(self.langue, T::Vide)));
                }
            });
        self.fenetre_editeur(&ctx);
        self.dialogues(&ctx);
        if let Some((m, _)) = &self.message {
            let m = m.clone();
            egui::Area::new("message".into())
                .anchor(egui::Align2::CENTER_BOTTOM, vec2(0.0, -24.0))
                .show(&ctx, |ui| {
                    Frame::new()
                        .fill(teinte::GRAPHITE)
                        .corner_radius(8)
                        .inner_margin(Margin::symmetric(16, 10))
                        .show(ui, |ui| {
                            ui.set_max_width(520.0);
                            ui.label(egui::RichText::new(m).color(Color32::WHITE));
                        });
                });
        }
    }

    fn bouton_rail(&mut self, ui: &mut egui::Ui, icone: Icone, nom: T, actif: bool) -> egui::Response {
        let (r, rep) = ui.allocate_exact_size(vec2(68.0, 56.0), Sense::click());
        let rep = rep.on_hover_text(t(self.langue, nom));
        rep.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Button, true, actif, t(self.langue, nom))
        });
        let p = ui.painter();
        // L'outil actif : une pastille noire, l'icône en blanc.
        if actif {
            p.rect_filled(r.shrink2(vec2(10.0, 5.0)), 14.0, teinte::GRAPHITE);
        } else if rep.hovered() {
            p.rect_filled(r.shrink2(vec2(10.0, 5.0)), 14.0, teinte::SURVOL);
        }
        let c = if actif { Color32::WHITE } else { teinte::GRAPHITE };
        icones::dessiner(
            p,
            egui::Rect::from_center_size(r.center(), vec2(30.0, 30.0)),
            icone,
            c,
        );
        if rep.has_focus() {
            p.rect_stroke(
                r.shrink(3.0),
                10.0,
                Stroke::new(2.0, teinte::ENCRE),
                egui::StrokeKind::Inside,
            );
        }
        rep
    }

    fn rail(&mut self, ui: &mut egui::Ui) {
        let r = ui.max_rect();
        // Un filet sépare le rail de la feuille.
        ui.painter()
            .vline(r.right() - 0.5, r.y_range(), Stroke::new(1.0, teinte::FILET));
        ui.spacing_mut().item_spacing.y = 2.0;
        ui.add_space(6.0);
        if self
            .bouton_rail(ui, Icone::Coffre, T::Coffre, self.tiroir)
            .clicked()
        {
            self.tiroir = !self.tiroir;
        }
        ui.add_space(10.0);
        for (icone, nom, outil) in [
            (Icone::Ecrire, T::Ecrire, Outil::Ecrire),
            (Icone::Formes, T::Formes, Outil::Formes),
            (Icone::Clavier, T::Clavier, Outil::Clavier),
            (Icone::Selection, T::Selection, Outil::Selection),
            (Icone::Gomme, T::Gomme, Outil::Gomme),
        ] {
            if self.bouton_rail(ui, icone, nom, self.outil == outil).clicked() {
                self.outil = outil;
                if outil == Outil::Clavier && self.editeur.is_none() {
                    self.nouveau_texte_clavier();
                }
            }
        }
        ui.add_space(10.0);
        // La couleur et l'épaisseur courantes, en une pastille.
        let (cr, crep) = ui.allocate_exact_size(vec2(68.0, 50.0), Sense::click());
        let crep = crep.on_hover_text(t(self.langue, T::Couleur));
        let rayon = 7.0 + self.epaisseur * 1.6;
        ui.painter()
            .circle_filled(cr.center(), rayon.min(18.0), crate::peinture::c32(self.couleur));
        if self.surligneur {
            ui.painter()
                .circle_stroke(cr.center(), 20.0, Stroke::new(3.0, teinte::FILET));
        }
        egui::Popup::menu(&crep).show(|ui| self.menu_couleur(ui));
        if self
            .bouton_rail(ui, Icone::AuPropre, T::AuPropre, false)
            .clicked()
        {
            self.au_propre_force();
        }
        if self.bouton_rail(ui, Icone::Diapos, T::Diapos, false).clicked() && self.ouverte.is_some() {
            self.presentation = Some(self.page_vue);
        }
        let plus = self.bouton_rail(ui, Icone::Plus, T::Plus, false);
        egui::Popup::menu(&plus).show(|ui| self.menu_plus(ui));
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            if self
                .bouton_rail(ui, Icone::Retablir, T::Retablir, false)
                .clicked()
            {
                self.retablir();
            }
            if self.bouton_rail(ui, Icone::Annuler, T::Annuler, false).clicked() {
                self.annuler();
            }
        });
    }

    fn menu_couleur(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(220.0);
        ui.horizontal_wrapped(|ui| {
            for c in PALETTE {
                let (r, rep) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::click());
                ui.painter()
                    .circle_filled(r.center(), 15.0, crate::peinture::c32(c));
                if c == self.couleur {
                    ui.painter()
                        .circle_stroke(r.center(), 19.0, Stroke::new(2.0, teinte::GRAPHITE));
                }
                if rep.clicked() {
                    self.couleur = c;
                }
            }
        });
        ui.add(egui::Slider::new(&mut self.epaisseur, 1.0..=8.0).text(t(self.langue, T::Epaisseur)));
        ui.checkbox(&mut self.surligneur, t(self.langue, T::Surligneur));
        ui.checkbox(&mut self.rempli, t(self.langue, T::Remplir));
        // Appliquer à la sélection.
        if !self.selection.is_empty() && ui.button(t(self.langue, T::Valider)).clicked() {
            let (c, page, sel) = (self.couleur, self.page_sel, self.selection.clone());
            self.memoriser();
            if let Some(p) = self.pages().and_then(|p| p.get_mut(page)) {
                for i in sel {
                    match p.elements.get_mut(i) {
                        Some(Element::Encre(t)) => t.couleur = c,
                        Some(Element::Texte(t)) => t.couleur = c,
                        Some(Element::Forme(f)) => f.couleur = c,
                        _ => {}
                    }
                }
            }
            self.modifie();
        }
    }

    fn menu_plus(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(280.0);
        if ui.button(t(self.langue, T::InsererImage)).clicked() {
            self.plateforme.demander_image();
        }
        ui.separator();
        if ui.button(t(self.langue, T::AjouterPage)).clicked() {
            let page = self.page_vue;
            self.memoriser();
            if let Some(o) = &mut self.ouverte {
                let fond = o.lecon.pages.get(page).map(|p| p.fond).unwrap_or_default();
                o.lecon.pages.insert(
                    page + 1,
                    Page {
                        elements: vec![],
                        fond,
                    },
                );
            }
            self.modifie();
        }
        let n = self.ouverte.as_ref().map_or(0, |o| o.lecon.pages.len());
        if n > 1 && ui.button(t(self.langue, T::SupprimerPage)).clicked() {
            let page = self.page_vue.min(n - 1);
            self.memoriser();
            if let Some(o) = &mut self.ouverte {
                o.lecon.pages.remove(page);
            }
            self.modifie();
        }
        ui.label(egui::RichText::new(t(self.langue, T::Fond)).color(teinte::DOUX));
        ui.horizontal_wrapped(|ui| {
            for (f, nom) in [
                (fano_core::Fond::Blanc, T::FondBlanc),
                (fano_core::Fond::Lignes, T::FondLignes),
                (fano_core::Fond::Carreaux, T::FondCarreaux),
                (fano_core::Fond::Points, T::FondPoints),
            ] {
                let page = self.page_vue;
                let actuel = self
                    .ouverte
                    .as_ref()
                    .and_then(|o| o.lecon.pages.get(page))
                    .map(|p| p.fond);
                if ui
                    .selectable_label(actuel == Some(f), t(self.langue, nom))
                    .clicked()
                {
                    self.memoriser();
                    if let Some(p) = self.pages().and_then(|p| p.get_mut(page)) {
                        p.fond = f;
                    }
                    self.modifie();
                }
            }
        });
        ui.separator();
        if ui.button(t(self.langue, T::ExporterPdf)).clicked() {
            self.exporter("pdf");
        }
        if ui.button(t(self.langue, T::ExporterPng)).clicked() {
            self.exporter("png");
        }
        if ui.button(t(self.langue, T::ExporterMd)).clicked() {
            self.exporter("md");
        }
        ui.separator();
        ui.label(egui::RichText::new(t(self.langue, T::Reglages)).color(teinte::DOUX));
        let avant = self.reglages.clone();
        ui.checkbox(&mut self.reglages.doigt_ecrit, t(self.langue, T::DoigtEcrit));
        ui.checkbox(&mut self.reglages.au_propre_auto, t(self.langue, T::AuPropreAuto));
        ui.add(
            egui::Slider::new(&mut self.reglages.delai_ms, 400..=4000)
                .suffix(" ms")
                .text(t(self.langue, T::Delai)),
        );
        ui.horizontal(|ui| {
            ui.label(t(self.langue, T::Langue));
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
        ui.checkbox(&mut self.reglages.maj_auto, t(self.langue, T::MajAuto));
        if ui.button(t(self.langue, T::ChercherMaj)).clicked() {
            self.chercher_maj(&ui.ctx().clone(), true);
        }
        if self.reglages != avant {
            self.reglages.ecrire(&self.dossier);
        }
        ui.separator();
        ui.label(
            egui::RichText::new(format!(
                "Fanoratana {} · GPL-3.0-or-later",
                env!("CARGO_PKG_VERSION")
            ))
            .small()
            .color(teinte::DOUX),
        );
    }

    fn exporter(&mut self, quoi: &str) {
        self.enregistrer();
        let Some(o) = &self.ouverte else { return };
        let base = fano_core::coffre::nom_sur(&o.lecon.titre);
        let r = match quoi {
            "pdf" => fano_render::export::pdf_lecon(&o.lecon, &self.coffre.racine).and_then(|octets| {
                self.plateforme
                    .enregistrer(&format!("{base}.pdf"), "application/pdf", octets)
            }),
            "png" => {
                let mut dernier = Ok(None);
                for i in 0..o.lecon.pages.len() {
                    let nom = if o.lecon.pages.len() == 1 {
                        format!("{base}.png")
                    } else {
                        format!("{base}-{}.png", i + 1)
                    };
                    dernier = fano_render::export::png_page(&o.lecon, i, &self.coffre.racine, 2.0)
                        .and_then(|octets| self.plateforme.enregistrer(&nom, "image/png", octets));
                    if !matches!(dernier, Ok(Some(_))) {
                        break;
                    }
                }
                dernier
            }
            _ => self.plateforme.enregistrer(
                &format!("{base}.md"),
                "text/markdown",
                o.lecon.markdown().into_bytes(),
            ),
        };
        match r {
            Ok(Some(ou)) => {
                let m = format!("{} {ou}", t(self.langue, T::Exporte));
                self.dire(m);
            }
            Ok(None) => {}
            Err(e) => {
                let m = format!("{} {e}", t(self.langue, T::Erreur));
                self.dire(m);
            }
        }
    }

    /// Le mode présentation : une page à la fois, plein écran ; toucher à droite avance.
    fn presenter(&mut self, ui: &mut egui::Ui, i: usize) {
        let Some(o) = &self.ouverte else {
            self.presentation = None;
            return;
        };
        let n = o.lecon.pages.len();
        let i = i.min(n.saturating_sub(1));
        let format = o.lecon.format;
        let ecran = ui.max_rect();
        let rep = ui.allocate_rect(ecran, Sense::click());
        ui.painter().rect_filled(ecran, 0.0, Color32::BLACK);
        let (w, h) = format.taille();
        let zoom = (ecran.width() / w).min(ecran.height() / h);
        let origine = ecran.center() - vec2(w * zoom / 2.0, h * zoom / 2.0);
        let repere = crate::peinture::Repere { origine, zoom };
        let page = o.lecon.pages[i].clone();
        crate::peinture::page(
            ui.painter(),
            &repere,
            &page,
            format,
            &mut self.images,
            &self.coffre.racine,
        );
        let mut aller: i64 = 0;
        if rep.clicked() {
            if let Some(p) = rep.interact_pointer_pos() {
                aller = if p.x > ecran.center().x { 1 } else { -1 };
            }
        }
        let quitter = ui.input_mut(|inp| {
            use egui::{Key, Modifiers};
            if inp.consume_key(Modifiers::NONE, Key::ArrowRight)
                || inp.consume_key(Modifiers::NONE, Key::PageDown)
                || inp.consume_key(Modifiers::NONE, Key::Space)
            {
                aller = 1;
            }
            if inp.consume_key(Modifiers::NONE, Key::ArrowLeft)
                || inp.consume_key(Modifiers::NONE, Key::PageUp)
            {
                aller = -1;
            }
            inp.consume_key(Modifiers::NONE, Key::Escape)
        });
        let bouton = egui::Rect::from_min_size(ecran.right_top() + vec2(-64.0, 12.0), vec2(52.0, 52.0));
        let fermer = ui.put(
            bouton,
            egui::Button::new(egui::RichText::new("✕").size(22.0).color(Color32::WHITE))
                .fill(Color32::from_white_alpha(20)),
        );
        let fermer = fermer.on_hover_text(t(self.langue, T::Quitter));
        ui.painter().text(
            ecran.center_bottom() + vec2(0.0, -16.0),
            egui::Align2::CENTER_BOTTOM,
            format!("{} / {n}", i + 1),
            egui::FontId::proportional(15.0),
            Color32::from_white_alpha(140),
        );
        if quitter || fermer.clicked() {
            self.page_vue = i;
            self.presentation = None;
        } else {
            let j = (i as i64 + aller).clamp(0, n as i64 - 1) as usize;
            self.presentation = Some(j);
        }
    }
}
