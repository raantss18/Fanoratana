//! L'état de l'app et la mise en place de l'écran.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use egui::{Color32, Frame, Margin, Sense, vec2};
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

/// Comment lire l'écriture : texte, formule (LaTeX), ou deviner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeLecture {
    Auto,
    Texte,
    Formule,
}

pub fn icone_outil(o: Outil) -> Icone {
    match o {
        Outil::Ecrire => Icone::Ecrire,
        Outil::Formes => Icone::Formes,
        Outil::Gomme => Icone::Gomme,
        Outil::Selection => Icone::Selection,
        Outil::Clavier => Icone::Clavier,
    }
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
    /// Un trait tracé sous les mots : il les souligne au lieu d'être lu (il est dans `traits`).
    pub souligne: bool,
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
    pub(crate) mode_lecture: ModeLecture,
    pub(crate) tableau_taille: (usize, usize),
    /// Téléchargement du modèle des formules : part faite (0..1), ou erreur.
    /// La partie visible de la page vue : (haut, hauteur), en coordonnées de page.
    pub(crate) vue_page: (f32, f32),
    pub(crate) outils_ouverts: bool,
    pub(crate) reglages_ouverts: bool,
    pub(crate) decalage_haut: f32,
    /// Un point de page à faire venir à l'écran (après une insertion hors de vue).
    pub(crate) montrer: Option<(usize, f32)>,
    /// La fin de page a déjà été signalée pour cette page.
    pub(crate) page_proposee: bool,
    pub(crate) modele_formules: std::sync::Arc<std::sync::Mutex<Option<Result<f32, String>>>>,
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
        let lecteur = Some(Lecteur::demarrer(
            dossier.join("modeles").join("pix2text-mfr"),
            move || reveil.request_repaint(),
        ));
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
            mode_lecture: ModeLecture::Auto,
            tableau_taille: (3, 3),
            modele_formules: Default::default(),
            vue_page: (0.0, 0.0),
            outils_ouverts: false,
            reglages_ouverts: false,
            decalage_haut: 0.0,
            montrer: None,
            page_proposee: false,
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
        if self
            .ouverte
            .as_ref()
            .and_then(|o| o.lecon.pages.get(page))
            .is_none()
        {
            return;
        }
        let pos = fano_core::Point::new(72.0, self.bas_de_page(page) + 16.0);
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
            ..Default::default()
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
        egui::CentralPanel::default()
            .frame(Frame::new().fill(teinte::BUREAU))
            .show(ui, |ui| {
                let avant = ui.cursor().top();
                self.bandeau_maj(ui);
                self.decalage_haut = ui.cursor().top() - avant;
                if self.ouverte.is_some() {
                    self.toile(ui);
                } else {
                    ui.centered_and_justified(|ui| ui.label(t(self.langue, T::Vide)));
                }
            });
        self.coins(&ctx);
        self.fenetre_editeur(&ctx);
        self.dialogues(&ctx);
        if let Some((m, _)) = &self.message {
            let m = m.clone();
            egui::Area::new("message".into())
                .anchor(egui::Align2::CENTER_BOTTOM, vec2(0.0, -84.0))
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

    pub(crate) fn menu_inserer(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(280.0);
        let l = self.langue;
        if crate::widgets::ligne(ui, Some(Icone::Clavier), t(l, T::InsererTexte), false).clicked() {
            self.outil = Outil::Clavier;
            self.nouveau_texte_clavier();
            ui.close();
        }
        // Le tableau : taille choisie ici, puis posé sous ce qui est écrit.
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            icones::dessiner(
                ui.painter(),
                egui::Rect::from_center_size(ui.cursor().min + vec2(13.0, 22.0), vec2(26.0, 26.0)),
                Icone::Tableau,
                teinte::GRAPHITE,
            );
            ui.add_space(38.0);
            ui.add(
                egui::DragValue::new(&mut self.tableau_taille.0)
                    .range(1..=20)
                    .suffix(format!(" {}", t(l, T::Lignes))),
            );
            ui.label("×");
            ui.add(
                egui::DragValue::new(&mut self.tableau_taille.1)
                    .range(1..=10)
                    .suffix(format!(" {}", t(l, T::Colonnes))),
            );
        });
        if crate::widgets::ligne(ui, None, t(l, T::InsererTableau), false).clicked() {
            self.inserer_tableau();
            ui.close();
        }
        if crate::widgets::ligne(ui, Some(Icone::Graphe), t(l, T::InsererCourbe), false).clicked() {
            self.inserer_graphe(vec!["f(x)=x^2".into()]);
            ui.close();
        }
        if crate::widgets::ligne(ui, Some(Icone::Image), t(l, T::InsererImage), false).clicked() {
            self.plateforme.demander_image();
            ui.close();
        }
        if crate::widgets::ligne(ui, Some(Icone::Page), t(l, T::AjouterPage), false).clicked() {
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
            ui.close();
        }
    }

    /// Une page de plus à la fin, et la feuille défile jusqu'à elle.
    pub(crate) fn ajouter_page_fin(&mut self) {
        self.memoriser();
        let n = if let Some(o) = &mut self.ouverte {
            let fond = o.lecon.pages.last().map(|p| p.fond).unwrap_or_default();
            o.lecon.pages.push(Page {
                elements: vec![],
                fond,
            });
            o.lecon.pages.len()
        } else {
            return;
        };
        self.page_proposee = false;
        self.montrer = Some((n - 1, 0.0));
        self.modifie();
    }

    pub(crate) fn dossier_formules(&self) -> PathBuf {
        self.dossier.join("modeles").join("pix2text-mfr")
    }

    /// Le modèle des formules : présent, en cours de téléchargement, ou un bouton pour le télécharger.
    pub(crate) fn reglage_modele_formules(&mut self, ui: &mut egui::Ui) {
        let l = self.langue;
        let dossier = self.dossier_formules();
        if fano_reco::formules::present(&dossier) {
            ui.label(egui::RichText::new(t(l, T::ModelePresent)).color(teinte::DOUX));
            return;
        }
        let etat = self.modele_formules.lock().unwrap().clone();
        match etat {
            Some(Ok(p)) if p < 1.0 => {
                ui.add(egui::ProgressBar::new(p).text(format!("{:.0} %", p * 100.0)));
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(250));
            }
            autre => {
                if let Some(Err(e)) = autre {
                    ui.label(egui::RichText::new(format!("{} {e}", t(l, T::Erreur))).color(teinte::DANGER));
                }
                if ui.button(t(l, T::TelechargerModele)).clicked() {
                    let etat = self.modele_formules.clone();
                    *etat.lock().unwrap() = Some(Ok(0.0));
                    let ctx = ui.ctx().clone();
                    std::thread::spawn(move || {
                        let total: u64 = fano_reco::formules::FICHIERS.iter().map(|f| f.1).sum();
                        let mut deja = 0u64;
                        for (nom, taille, sha) in fano_reco::formules::FICHIERS {
                            let dest = dossier.join(nom);
                            if std::fs::metadata(&dest).is_ok_and(|m| m.len() == taille) {
                                deja += taille;
                                continue;
                            }
                            let e2 = etat.clone();
                            let base = deja;
                            let r = fano_maj::telecharger_fichier(
                                &fano_reco::formules::url(nom),
                                &dest,
                                taille,
                                sha,
                                move |n| {
                                    *e2.lock().unwrap() = Some(Ok((base + n) as f32 / total as f32 * 0.999));
                                },
                            );
                            if let Err(e) = r {
                                *etat.lock().unwrap() = Some(Err(e));
                                ctx.request_repaint();
                                return;
                            }
                            deja += taille;
                        }
                        *etat.lock().unwrap() = Some(Ok(1.0));
                        ctx.request_repaint();
                    });
                }
            }
        }
    }

    /// Un tableau neuf, sous ce qui est déjà sur la page vue.
    pub(crate) fn inserer_tableau(&mut self) {
        let page = self.page_vue;
        let Some(o) = &self.ouverte else { return };
        let (pw, _) = o.lecon.format.taille();
        let (l, c) = self.tableau_taille;
        let y = self.place_libre(page, l as f32 * 40.0);
        let tb = fano_core::Tableau::nouveau(fano_core::Point::new(72.0, y), l, c, pw - 72.0 - 40.0);
        self.memoriser();
        if let Some(p) = self.pages().and_then(|p| p.get_mut(page)) {
            p.elements.push(Element::Tableau(tb));
            self.selection = vec![p.elements.len() - 1];
            self.page_sel = page;
        }
        self.modifie();
    }

    /// Une courbe façon GeoGebra, sous ce qui est déjà sur la page vue.
    pub(crate) fn inserer_graphe(&mut self, definitions: Vec<String>) {
        let page = self.page_vue;
        let Some(o) = &self.ouverte else { return };
        let (pw, _) = o.lecon.format.taille();
        let (xmin, xmax, ymin, ymax) = fano_render::scene::bornes_auto(&definitions);
        let w = (pw - 72.0 - 40.0).min(420.0);
        let y = self.place_libre(page, w * 0.7);
        let g = fano_core::Graphe {
            rect: fano_core::Rect::from_min_size(fano_core::Point::new(72.0, y), w, w * 0.7),
            definitions,
            xmin,
            xmax,
            ymin,
            ymax,
        };
        self.memoriser();
        if let Some(p) = self.pages().and_then(|p| p.get_mut(page)) {
            p.elements.push(Element::Graphe(g));
            let i = p.elements.len() - 1;
            self.page_sel = page;
            self.selection = vec![i];
        }
        self.modifie();
    }

    /// Où poser un nouvel élément de hauteur `h` : un espace libre dans ce qu'on voit, sinon sous le
    /// contenu, et la feuille défile jusqu'à lui. Jamais par-dessus ce qui est écrit.
    pub(crate) fn place_libre(&mut self, page: usize, h: f32) -> f32 {
        let (haut, hauteur) = self.vue_page;
        let mut occupe: Vec<(f32, f32)> = self
            .ouverte
            .as_ref()
            .and_then(|o| o.lecon.pages.get(page))
            .map(|p| {
                p.elements
                    .iter()
                    .map(|e| {
                        let b = fano_render::scene::bbox(e);
                        (b.min.y - 12.0, b.max.y + 12.0)
                    })
                    .collect()
            })
            .unwrap_or_default();
        occupe.sort_by(|a, b| a.0.total_cmp(&b.0));
        if page == self.page_vue && hauteur > 0.0 {
            let mut y = (haut + 24.0).max(40.0);
            for (debut, fin) in &occupe {
                if *fin < y {
                    continue;
                }
                if *debut - y >= h {
                    break;
                }
                y = y.max(*fin);
            }
            if y + h <= haut + hauteur - 24.0 {
                return y;
            }
        }
        let y = self.bas_contenu(page) + 20.0;
        self.montrer = Some((page, y));
        y
    }

    pub(crate) fn bas_de_page(&mut self, page: usize) -> f32 {
        self.place_libre(page, 60.0)
    }

    pub(crate) fn bas_contenu(&self, page: usize) -> f32 {
        self.ouverte
            .as_ref()
            .and_then(|o| o.lecon.pages.get(page))
            .map(|p| {
                p.elements
                    .iter()
                    .map(|e| fano_render::scene::bbox(e).max.y)
                    .fold(48.0_f32, f32::max)
            })
            .unwrap_or(48.0)
    }

    pub(crate) fn exporter(&mut self, quoi: &str) {
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
