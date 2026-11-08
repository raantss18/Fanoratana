//! La toile : les pages l'une sous l'autre, les gestes, et la mise au propre.
//!
//! - Le stylet et la souris écrivent toujours ; le doigt écrit aussi (réglage), deux doigts
//!   déplacent et zooment.
//! - Fin d'un trait : rature ? forme (outil Formes, ou stylet tenu immobile au bout) ? sinon encre,
//!   mise en attente de lecture.
//! - Après la pause, l'encre en attente est regroupée en lignes ; chaque ligne est lue ; une lecture
//!   sûre remplace l'encre par le texte au propre (l'encre reste gardée dans `source`).

use egui::{Color32, Pos2, Sense, Stroke, vec2};
use fano_core::{Element, Forme, FormeKind, Point, PointEncre, Rect, Texte, Trait};

use crate::app::{EnLecture, Fanoratana, Geste, Outil};
use crate::editeur::Editeur;
use crate::peinture::{self, Repere};
use crate::plateforme::Pointeur;
use crate::teinte;
use crate::textes::{T, t};

const ECART: f32 = 28.0;
/// La feuille commence sous les boutons du haut (ils ne cachent jamais le haut de la page au repos).
const HAUT: f32 = 76.0;
/// La carte « Ajouter une page » sous la dernière page.
const CARTE: f32 = 120.0;

/// Ce que devient un trait de l'outil Formes.
enum Issue {
    /// Rien de reconnu : il reste de l'encre.
    Rien,
    /// C'était la pointe d'un segment, devenu flèche.
    Fleche,
    Forme(Forme),
}

impl Fanoratana {
    fn reperes(&self, zone: egui::Rect) -> Vec<Repere> {
        let Some(o) = &self.ouverte else { return vec![] };
        let (w, h) = o.lecon.format.taille();
        let x0 = zone.center().x - w * self.zoom / 2.0 + self.defil.x;
        (0..o.lecon.pages.len())
            .map(|i| Repere {
                origine: Pos2::new(
                    x0,
                    zone.top() + HAUT + self.defil.y + i as f32 * (h + ECART) * self.zoom,
                ),
                zoom: self.zoom,
            })
            .collect()
    }

    /// La page sous un point d'écran (la plus proche verticalement).
    fn page_sous(&self, reperes: &[Repere], e: Pos2) -> Option<(usize, Point)> {
        let o = self.ouverte.as_ref()?;
        let (_, h) = o.lecon.format.taille();
        reperes
            .iter()
            .enumerate()
            .min_by(|a, b| {
                let da = (a.1.page(e).y - h / 2.0).abs();
                let db = (b.1.page(e).y - h / 2.0).abs();
                da.total_cmp(&db)
            })
            .map(|(i, r)| (i, r.page(e)))
    }

    pub(crate) fn toile(&mut self, ui: &mut egui::Ui) {
        // La place qui reste sous le bandeau des mises à jour (la feuille ne le recouvre jamais).
        let zone = ui.available_rect_before_wrap();
        let format = self.ouverte.as_ref().map(|o| o.lecon.format).unwrap_or_default();
        let (w, h) = format.taille();
        // Rotation, fenêtre flottante, écran partagé : la feuille se recadre.
        if (zone.width() - self.largeur_vue).abs() > 2.0 {
            if self.largeur_vue > 0.0 {
                let ancien = ((self.largeur_vue - 16.0) / w).clamp(0.3, 2.5);
                if (self.zoom - ancien).abs() < 1e-3 {
                    self.recadrer = true;
                }
            }
            self.largeur_vue = zone.width();
        }
        if self.recadrer {
            self.zoom = ((zone.width() - 16.0) / w).clamp(0.3, 2.5);
            self.defil = egui::Vec2::ZERO;
            self.recadrer = false;
        }
        let rep = ui.allocate_rect(zone, Sense::click_and_drag());
        self.gestes(ui, zone, &rep);
        // Défilement borné : on ne perd jamais la feuille.
        let n = self.ouverte.as_ref().map_or(1, |o| o.lecon.pages.len()) as f32;
        let total = n * (h + ECART) * self.zoom + HAUT + CARTE * self.zoom + 80.0;
        self.defil.y = self.defil.y.clamp(-(total - zone.height() * 0.5).max(0.0), 0.0);
        // Gauche-droite : la page est ajustée à la largeur et bloquée sur ses bords ; zoomée, on va
        // jusqu'à ses bords, jamais au-delà.
        let largeur = w * self.zoom;
        let jeu = ((largeur - zone.width()) / 2.0 + 8.0).max(0.0);
        self.defil.x = if largeur <= zone.width() - 16.0 {
            0.0
        } else {
            self.defil.x.clamp(-jeu, jeu)
        };

        // Faire venir à l'écran ce qui vient d'être inséré plus bas.
        if let Some((p, y)) = self.montrer.take() {
            if let Some(r) = self.reperes(zone).get(p) {
                let actuel = r.ecran(Point::new(0.0, y)).y;
                self.defil.y -= actuel - (zone.top() + zone.height() * 0.3);
            }
        }
        let reperes = self.reperes(zone);
        let painter = ui.painter_at(zone);
        // La page vue : celle qui occupe le milieu de l'écran.
        if let Some((i, _)) = self.page_sous(&reperes, zone.center()) {
            self.page_vue = i;
            if let Some(r) = reperes.get(i) {
                let haut = r.page(zone.left_top()).y.max(0.0);
                self.vue_page = (haut, zone.height() / self.zoom);
            }
        }
        let coffre = self.coffre.racine.clone();
        if let Some(o) = &self.ouverte {
            for (i, r) in reperes.iter().enumerate() {
                let rect = r.rect(Rect::from_min_size(Point::new(0.0, 0.0), w, h));
                if !rect.intersects(zone) {
                    continue;
                }
                peinture::page(&painter, r, &o.lecon.pages[i], format, &mut self.images, &coffre);
            }
        }
        // Sous la dernière page : une carte pour en ajouter une.
        if let Some(r) = reperes.last() {
            let carte = r.rect(Rect::from_min_size(
                Point::new(0.0, h + ECART * 0.6),
                w,
                CARTE - ECART,
            ));
            if carte.intersects(zone) {
                painter.add(egui::Shape::dashed_line(
                    &rect_points(carte),
                    Stroke::new(1.5, teinte::DOUX),
                    8.0,
                    6.0,
                ));
                painter.text(
                    carte.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("+  {}", t(self.langue, T::AjouterPage)),
                    egui::FontId::proportional(18.0),
                    teinte::GRAPHITE,
                );
                let rep = ui.interact(carte, ui.id().with("ajouter-page"), Sense::click());
                if rep.clicked() {
                    self.ajouter_page_fin();
                }
            }
        }
        // Une page vide dit quoi faire.
        if let Some(o) = &self.ouverte {
            for (i, r) in reperes.iter().enumerate() {
                if o.lecon.pages[i].elements.is_empty() && self.geste.is_none() {
                    let pos = r.ecran(Point::new(72.0, 112.0));
                    let galley = painter.layout(
                        t(self.langue, T::Bienvenue).to_owned(),
                        egui::FontId::proportional(16.0),
                        teinte::DOUX,
                        (w - 140.0) * self.zoom,
                    );
                    painter.galley(pos, galley, teinte::DOUX);
                }
            }
        }
        // Le trait en cours.
        if let Some(Geste::Trait { page, trait_, .. }) = &self.geste {
            if let Some(r) = reperes.get(*page) {
                peinture::encre(&painter, r, trait_);
            }
        }
        if !self.lectures.is_empty() {
            painter.text(
                zone.right_top() + vec2(-16.0, 12.0),
                egui::Align2::RIGHT_TOP,
                t(self.langue, T::Lecture),
                egui::FontId::proportional(14.0),
                teinte::DOUX,
            );
        }
        // Sélection.
        if let Some(r) = reperes.get(self.page_sel) {
            if let Some(b) = self.bbox_selection() {
                painter.add(egui::Shape::dashed_line(
                    &rect_points(r.rect(b.expand(6.0))),
                    Stroke::new(1.5, teinte::ENCRE),
                    6.0,
                    4.0,
                ));
            }
        }
        if let Some(Geste::RectSel { page, debut, fin }) = &self.geste {
            if let Some(r) = reperes.get(*page) {
                let b = Rect::from_points([*debut, *fin]);
                painter.rect(
                    r.rect(b),
                    2.0,
                    Color32::from_black_alpha(10),
                    Stroke::new(1.0, teinte::ENCRE),
                    egui::StrokeKind::Middle,
                );
            }
        }
        self.barre_contexte(ui, zone);
        // Le numéro de page, discret, en bas au centre (le titre est dans le coin haut gauche).
        if let Some(o) = &self.ouverte {
            let n = o.lecon.pages.len();
            if n > 1 {
                painter.text(
                    zone.center_bottom() + vec2(0.0, -26.0),
                    egui::Align2::CENTER_BOTTOM,
                    format!("{} / {n}", self.page_vue + 1),
                    egui::FontId::proportional(14.0),
                    teinte::DOUX,
                );
            }
        }
    }

    /// La barre contextuelle : seulement quand quelque chose est sélectionné.
    fn barre_contexte(&mut self, ui: &mut egui::Ui, zone: egui::Rect) {
        if self.selection.is_empty() || self.geste.is_some() {
            return;
        }
        let l = self.langue;
        let page = self.page_sel;
        let seul = (self.selection.len() == 1).then(|| self.selection[0]);
        let element = seul.and_then(|i| {
            self.ouverte
                .as_ref()?
                .lecon
                .pages
                .get(page)?
                .elements
                .get(i)
                .cloned()
        });
        let mut action: Option<Box<dyn FnOnce(&mut fano_core::Tableau)>> = None;
        let mut supprimer = false;
        let mut propre = false;
        let mut modifier_graphe = None;
        egui::Area::new("contexte".into())
            .fixed_pos(zone.center_top() + vec2(-220.0, 34.0))
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::window(ui.style())
                    .inner_margin(egui::Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if let Some(Element::Tableau(tb)) = &element {
                                if ui.button(t(l, T::AjouterLigne)).clicked() {
                                    action = Some(Box::new(|t| t.ajouter_ligne()));
                                }
                                if ui.button(t(l, T::AjouterColonne)).clicked() {
                                    action = Some(Box::new(|t| t.ajouter_colonne()));
                                }
                                if ui.button(t(l, T::RetirerLigne)).clicked() {
                                    action = Some(Box::new(|t| t.retirer_ligne()));
                                }
                                if ui.button(t(l, T::RetirerColonne)).clicked() {
                                    action = Some(Box::new(|t| t.retirer_colonne()));
                                }
                                let mut entete = tb.entete;
                                if ui.toggle_value(&mut entete, t(l, T::EnTete)).clicked() {
                                    action = Some(Box::new(move |t| t.entete = entete));
                                }
                            }
                            if let Some(Element::Graphe(g)) = &element {
                                if ui.button(t(l, T::Tracer)).clicked() {
                                    modifier_graphe = Some(g.definitions.clone());
                                }
                            }
                            if element.as_ref().is_none_or(|e| matches!(e, Element::Encre(_)))
                                && ui.button(t(l, T::AuPropre)).clicked()
                            {
                                propre = true;
                            }
                            if ui
                                .button(egui::RichText::new(t(l, T::Supprimer)).color(teinte::DANGER))
                                .clicked()
                            {
                                supprimer = true;
                            }
                        });
                    });
            });
        if let (Some(f), Some(i)) = (action, seul) {
            self.memoriser();
            if let Some(Element::Tableau(tb)) = self
                .pages()
                .and_then(|v| v.get_mut(page))
                .and_then(|p| p.elements.get_mut(i))
            {
                f(tb);
            }
            self.modifie();
        }
        if let (Some(defs), Some(i)) = (modifier_graphe, seul) {
            self.editeur = Some(Editeur::graphe(page, Some(i), &defs));
        }
        if propre {
            self.au_propre_force();
        }
        if supprimer {
            self.supprimer_selection();
        }
    }

    /// Tous les gestes de la toile.
    fn gestes(&mut self, ui: &mut egui::Ui, zone: egui::Rect, rep: &egui::Response) {
        let reperes = self.reperes(zone);
        let (multi, defil, zoom_delta, survol, appui, relache, pos, force, toucher) = ui.input(|i| {
            let mut force = None;
            let mut toucher = None;
            for e in &i.events {
                if let egui::Event::Touch { id, force: f, .. } = e {
                    toucher = Some(id.0);
                    force = *f;
                }
            }
            (
                i.multi_touch(),
                i.smooth_scroll_delta,
                i.zoom_delta(),
                i.pointer.hover_pos(),
                i.pointer.any_pressed(),
                i.pointer.any_released(),
                i.pointer.interact_pos(),
                force,
                toucher,
            )
        });
        if let Some(t) = toucher {
            self.toucher = Some(t);
        }
        // Deux doigts : déplacer et zoomer ; le trait commencé par le premier doigt est abandonné.
        if let Some(m) = multi {
            if matches!(self.geste, Some(Geste::Trait { .. })) {
                self.geste = None;
            }
            self.geste = Some(Geste::Defiler);
            self.zoomer(m.zoom_delta, m.center_pos, zone);
            self.defil += m.translation_delta;
            return;
        }
        if matches!(self.geste, Some(Geste::Defiler)) && !ui.input(|i| i.pointer.any_down()) {
            self.geste = None;
        }
        // Molette : défiler ; Ctrl + molette ou pincement du pavé : zoomer.
        if rep.hovered() {
            if zoom_delta != 1.0 {
                if let Some(p) = survol {
                    self.zoomer(zoom_delta, p, zone);
                }
            } else if defil != egui::Vec2::ZERO {
                self.defil += defil;
            }
        }
        let maintenant = self.maintenant;
        if appui && rep.hovered() && self.geste.is_none() {
            let Some(p) = pos else { return };
            let Some((page, pt)) = self.page_sous(&reperes, p) else {
                return;
            };
            // Hors des feuilles (entre deux pages, sur la carte « Ajouter une page ») : ni encre ni geste.
            let h_page = self.ouverte.as_ref().map_or(842.0, |o| o.lecon.format.taille().1);
            if (pt.y < -2.0 || pt.y > h_page + 2.0)
                && matches!(self.outil, Outil::Ecrire | Outil::Formes | Outil::Gomme)
            {
                return;
            }
            let pointeur = self.plateforme.pointeur(self.toucher);
            let doigt_deplace = pointeur == Pointeur::Doigt && !self.reglages.doigt_ecrit;
            self.geste = match self.outil {
                // Le doigt qui n'écrit pas déplace la page, quel que soit l'outil (le stylet fait l'outil).
                _ if doigt_deplace && self.outil != Outil::Clavier => Some(Geste::Defiler),
                Outil::Ecrire | Outil::Formes => Some(Geste::Trait {
                    page,
                    trait_: Trait {
                        points: vec![PointEncre {
                            x: pt.x,
                            y: pt.y,
                            p: force.unwrap_or(0.5),
                            t: 0,
                        }],
                        couleur: if self.surligneur {
                            fano_core::Couleur::SURLIGNEUR
                        } else {
                            self.couleur
                        },
                        epaisseur: if self.surligneur {
                            self.epaisseur * 6.0
                        } else {
                            self.epaisseur
                        },
                        surligneur: self.surligneur,
                    },
                    debut: maintenant,
                    surligneur: self.surligneur,
                }),
                Outil::Gomme => Some(Geste::Gomme {
                    page,
                    memorise: false,
                }),
                Outil::Selection => {
                    if page == self.page_sel
                        && self.bbox_selection().is_some_and(|b| b.expand(10.0).contains(pt))
                    {
                        Some(Geste::Deplacer {
                            page,
                            dernier: pt,
                            memorise: false,
                        })
                    } else {
                        self.selection.clear();
                        Some(Geste::RectSel {
                            page,
                            debut: pt,
                            fin: pt,
                        })
                    }
                }
                Outil::Clavier => None,
            };
        }
        // Mouvement.
        if let Some(p) = pos.filter(|_| ui.input(|i| i.pointer.any_down())) {
            match &mut self.geste {
                Some(Geste::Trait {
                    page, trait_, debut, ..
                }) => {
                    if let Some(r) = reperes.get(*page) {
                        let q = r.page(p);
                        let dernier = trait_.points.last().map(|d| d.pos()).unwrap_or(q);
                        if q.dist(dernier) * self.zoom >= 0.8 {
                            let t = ((maintenant - *debut) * 1000.0) as u32;
                            trait_.points.push(PointEncre {
                                x: q.x,
                                y: q.y,
                                p: force.unwrap_or(0.5),
                                t,
                            });
                        } else if let Some(d) = trait_.points.last_mut() {
                            // Immobile : on garde le temps (pour « tenir au bout » = forme).
                            d.t = d.t.max(0);
                        }
                    }
                }
                Some(Geste::Gomme { page, memorise }) => {
                    let (page, deja) = (*page, *memorise);
                    if let Some(r) = reperes.get(page) {
                        let q = r.page(p);
                        let rayon = 10.0 / self.zoom;
                        let touches = self.elements_sous(page, q, rayon);
                        if !touches.is_empty() {
                            if !deja {
                                self.memoriser();
                                if let Some(Geste::Gomme { memorise, .. }) = &mut self.geste {
                                    *memorise = true;
                                }
                            }
                            if let Some(pg) = self.pages().and_then(|v| v.get_mut(page)) {
                                for i in touches.into_iter().rev() {
                                    pg.elements.remove(i);
                                }
                            }
                            self.modifie();
                        }
                    }
                }
                Some(Geste::RectSel { page, fin, .. }) => {
                    if let Some(r) = reperes.get(*page) {
                        *fin = r.page(p);
                    }
                }
                Some(Geste::Deplacer {
                    page,
                    dernier,
                    memorise,
                }) => {
                    if let Some(r) = reperes.get(*page) {
                        let q = r.page(p);
                        let d = q - *dernier;
                        *dernier = q;
                        let (page, deja) = (*page, *memorise);
                        *memorise = true;
                        if !deja {
                            self.memoriser();
                        }
                        let sel = self.selection.clone();
                        if let Some(pg) = self.pages().and_then(|v| v.get_mut(page)) {
                            for i in sel {
                                if let Some(e) = pg.elements.get_mut(i) {
                                    e.translate(d);
                                }
                            }
                        }
                        self.modifie();
                    }
                }
                Some(Geste::Defiler) => {
                    self.defil += rep.drag_delta();
                }
                None => {}
            }
        }
        // Clic simple de l'outil Clavier : éditer le texte touché, ou en commencer un.
        if self.outil == Outil::Clavier && rep.clicked() {
            if let Some((page, pt)) = pos.and_then(|p| self.page_sous(&reperes, p)) {
                self.editer_ou_creer(page, pt);
            }
        }
        if relache {
            match self.geste.take() {
                Some(Geste::Trait {
                    page,
                    trait_,
                    debut,
                    surligneur,
                }) => self.finir_trait(page, trait_, maintenant - debut, surligneur),
                Some(Geste::RectSel { page, debut, fin }) => {
                    let b = Rect::from_points([debut, fin]);
                    self.page_sel = page;
                    self.selection = self
                        .ouverte
                        .as_ref()
                        .and_then(|o| o.lecon.pages.get(page))
                        .map(|p| {
                            p.elements
                                .iter()
                                .enumerate()
                                .filter(|(_, e)| {
                                    let eb = fano_render::scene::bbox(e);
                                    if b.width() < 3.0 && b.height() < 3.0 {
                                        eb.contains(debut)
                                    } else {
                                        b.intersects(&eb)
                                    }
                                })
                                .map(|(i, _)| i)
                                .collect()
                        })
                        .unwrap_or_default();
                }
                _ => {}
            }
        }
    }

    fn zoomer(&mut self, facteur: f32, centre: Pos2, zone: egui::Rect) {
        let ancien = self.zoom;
        self.zoom = (self.zoom * facteur).clamp(0.25, 5.0);
        let k = self.zoom / ancien;
        // Le point sous les doigts reste sous les doigts.
        let haut = Pos2::new(zone.center().x, zone.top() + HAUT);
        let rel = centre - haut - self.defil;
        self.defil += rel - rel * k;
    }

    /// Index des éléments d'une page touchés par un disque.
    fn elements_sous(&self, page: usize, q: Point, rayon: f32) -> Vec<usize> {
        let Some(p) = self.ouverte.as_ref().and_then(|o| o.lecon.pages.get(page)) else {
            return vec![];
        };
        p.elements
            .iter()
            .enumerate()
            .filter(|(_, e)| match e {
                Element::Encre(t) => {
                    t.bbox().expand(rayon + t.epaisseur).contains(q)
                        && t.points.iter().any(|pt| pt.pos().dist(q) < rayon + t.epaisseur)
                }
                e => fano_render::scene::bbox(e).expand(rayon * 0.5).contains(q),
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub(crate) fn bbox_selection(&self) -> Option<Rect> {
        let p = self.ouverte.as_ref()?.lecon.pages.get(self.page_sel)?;
        let r = self
            .selection
            .iter()
            .filter_map(|&i| p.elements.get(i))
            .map(fano_render::scene::bbox)
            .fold(Rect::NOTHING, Rect::union);
        r.is_valid().then_some(r)
    }

    /// L'éditeur de ce qui est sous le doigt (texte, case de tableau, courbe), s'il y a quelque chose.
    fn editeur_sous(&self, page: usize, pt: Point) -> Option<Editeur> {
        let p = self.ouverte.as_ref()?.lecon.pages.get(page)?;
        p.elements.iter().enumerate().rev().find_map(|(i, e)| match e {
            Element::Texte(t) if fano_render::scene::bbox(e).expand(6.0).contains(pt) => {
                Some(Editeur::existant(page, i, t.clone()))
            }
            Element::Tableau(tb) => tb
                .case(pt)
                .map(|(l, c)| Editeur::case(page, i, l, c, tb.cellules[l][c].clone())),
            Element::Graphe(g) if g.rect.contains(pt) => Some(Editeur::graphe(page, Some(i), &g.definitions)),
            _ => None,
        })
    }

    fn editer_ou_creer(&mut self, page: usize, pt: Point) {
        self.editeur = Some(
            self.editeur_sous(page, pt)
                .unwrap_or_else(|| Editeur::nouveau(page, self.texte_vide(Point::new(pt.x, pt.y - 14.0)))),
        );
    }

    /// La fin d'un trait.
    fn finir_trait(&mut self, page: usize, trait_: Trait, duree: f64, surligneur: bool) {
        let pts: Vec<Point> = trait_.points.iter().map(PointEncre::pos).collect();
        let longueur = trait_.longueur();
        // Un toucher bref sur un texte, une case ou une courbe : l'ouvrir au clavier.
        if longueur * self.zoom < 4.0 && duree < 0.35 && !surligneur {
            if let Some(ed) = self.editeur_sous(page, pts[0]) {
                self.editeur = Some(ed);
                return;
            }
        }
        if surligneur {
            self.memoriser();
            if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
                p.elements.push(Element::Encre(trait_));
            }
            self.modifie();
            return;
        }
        // Rature : allers-retours sur quelque chose → efface.
        let b = trait_.bbox();
        let sous: Vec<usize> = self
            .ouverte
            .as_ref()
            .and_then(|o| o.lecon.pages.get(page))
            .map(|p| {
                p.elements
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| {
                        let eb = fano_render::scene::bbox(e);
                        let inter = Rect {
                            min: Point::new(eb.min.x.max(b.min.x), eb.min.y.max(b.min.y)),
                            max: Point::new(eb.max.x.min(b.max.x), eb.max.y.min(b.max.y)),
                        };
                        inter.is_valid()
                            && inter.width() * inter.height() >= 0.35 * (eb.width() * eb.height()).max(1.0)
                    })
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default();
        if !sous.is_empty() && fano_ink::est_rature(&pts, 1.0) {
            self.memoriser();
            if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
                for &i in sous.iter().rev() {
                    p.elements.remove(i);
                }
            }
            self.attente
                .retain(|(pg, t)| *pg != page || !t.bbox().intersects(&b));
            self.modifie();
            return;
        }
        // Un trait plat sous un texte au propre le souligne (le trait n'est pas gardé).
        if let Some(i) = self.texte_sous_trait(page, &trait_) {
            self.memoriser();
            if let Some(Element::Texte(tx)) = self
                .pages()
                .and_then(|v| v.get_mut(page))
                .and_then(|p| p.elements.get_mut(i))
            {
                tx.souligne = true;
            }
            self.modifie();
            return;
        }
        // Forme : outil Formes, ou trait tenu immobile au bout (≥ 0,5 s).
        let tenu = trait_.points.len() >= 2 && {
            let fin = trait_.points.last().unwrap();
            let avant = trait_.points[trait_.points.len() - 2];
            duree - f64::from(fin.t) / 1000.0 >= 0.5 || (fin.t - avant.t) >= 500
        };
        if self.outil == Outil::Formes || tenu {
            match self.forme_de(page, &trait_, &pts) {
                Issue::Fleche => return,
                Issue::Forme(f) => return self.poser_forme(page, f),
                Issue::Rien => {}
            }
        }
        self.memoriser();
        if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
            p.elements.push(Element::Encre(trait_.clone()));
        }
        self.modifie();
        // On écrit en bas de la dernière page : la suivante est proposée.
        let derniere = self
            .ouverte
            .as_ref()
            .is_some_and(|o| page + 1 == o.lecon.pages.len());
        let h_page = self.ouverte.as_ref().map_or(842.0, |o| o.lecon.format.taille().1);
        if derniere && trait_.bbox().max.y > h_page * 0.88 && !self.page_proposee {
            self.page_proposee = true;
            let m = t(self.langue, T::FinDePage).to_owned();
            self.dire(m);
        }
        if self.outil == Outil::Ecrire {
            self.attente.push((page, trait_));
            self.derniere_ecriture = Some(self.maintenant);
        }
    }

    /// La forme reconnue (avec la règle de la pointe de flèche dessinée en second trait).
    fn forme_de(&mut self, page: usize, trait_: &Trait, pts: &[Point]) -> Issue {
        // Une pointe juste après un segment : il devient flèche.
        if let Some((pg, i, quand)) = self.dernier_segment {
            if pg == page && self.maintenant - quand < 4.0 {
                let seg = self
                    .ouverte
                    .as_ref()
                    .and_then(|o| o.lecon.pages.get(page))
                    .and_then(|p| p.elements.get(i))
                    .cloned();
                if let Some(Element::Forme(f)) = seg {
                    if f.kind == FormeKind::Segment {
                        let (a, b) = (f.points[0], f.points[1]);
                        let l = a.dist(b);
                        let mut fl = f.clone();
                        if fano_ink::formes::est_pointe(pts, b, l) {
                            fl.kind = FormeKind::Fleche;
                        } else if fano_ink::formes::est_pointe(pts, a, l) {
                            fl.kind = FormeKind::Fleche;
                            fl.points = vec![b, a];
                        }
                        if fl.kind == FormeKind::Fleche {
                            fl.source.push(trait_.clone());
                            self.memoriser();
                            if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
                                p.elements[i] = Element::Forme(fl);
                            }
                            self.modifie();
                            self.dernier_segment = None;
                            return Issue::Fleche;
                        }
                    }
                }
            }
        }
        let Some(r) = fano_ink::reconnaitre_forme(pts) else {
            return Issue::Rien;
        };
        Issue::Forme(Forme {
            kind: r.kind,
            points: r.points,
            couleur: trait_.couleur,
            epaisseur: trait_.epaisseur.max(1.6),
            rempli: self.rempli,
            graine: (trait_.points.len() as u32).wrapping_mul(2_654_435_761) ^ (self.maintenant as u32),
            etiquette: String::new(),
            source: vec![trait_.clone()],
        })
    }

    fn poser_forme(&mut self, page: usize, f: Forme) {
        self.memoriser();
        let kind = f.kind;
        if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
            p.elements.push(Element::Forme(f));
            let i = p.elements.len() - 1;
            self.dernier_segment = (kind == FormeKind::Segment).then_some((page, i, self.maintenant));
        }
        self.modifie();
        self.ajuster_graphes(page);
    }

    /// Envoie des traits à la lecture, ligne par ligne.
    pub(crate) fn lire_traits(&mut self, traits: Vec<(usize, Trait)>, forcee: bool) {
        if self.lecteur.is_none() {
            return;
        }
        let mut par_page: std::collections::BTreeMap<usize, Vec<Trait>> = Default::default();
        for (p, t) in traits {
            par_page.entry(p).or_default().push(t);
        }
        for (page, traits) in par_page {
            // 1ʳᵉ passe : les grands tracés isolés qui sont des formes (ronds, triangles, traits longs).
            let boites: Vec<Rect> = traits.iter().map(Trait::bbox).collect();
            // Une barre de fraction n'est pas un trait de dessin, même longue.
            let barres_fraction = fano_ink::barres_de_fraction(&boites);
            let mut formes: Vec<Trait> = Vec::new();
            for ligne in fano_ink::grouper_blocs(&boites) {
                let tous: Vec<Trait> = ligne.traits.iter().map(|&i| traits[i].clone()).collect();
                let bbox = tous.iter().fold(Rect::NOTHING, |r, t| r.union(t.bbox()));
                // Un trait sous des mots est leur soulignement, pas une ligne de dessin.
                formes.extend(
                    ligne
                        .traits
                        .iter()
                        .filter(|&&i| barres_fraction[i].is_none())
                        .map(|&i| &traits[i])
                        .filter(|t| forme_isolee(t, &tous).is_some() && !est_soulignement(t, &bbox, &tous))
                        .cloned(),
                );
            }
            if !formes.is_empty() {
                self.convertir_formes(page, &formes);
            }
            // 2ᵉ passe : les traits entre deux ronds sont des arêtes (graphe, arbre), ronds compris.
            let restants: Vec<Trait> = traits.iter().filter(|t| !formes.contains(t)).cloned().collect();
            let aretes: Vec<Trait> = restants
                .iter()
                .filter(|t| self.arete_entre_ronds(page, t))
                .cloned()
                .collect();
            if !aretes.is_empty() {
                self.convertir_formes(page, &aretes);
            }
            let traits: Vec<Trait> = restants.into_iter().filter(|t| !aretes.contains(t)).collect();
            // 3ᵉ passe : l'écriture, ligne par ligne.
            let boites: Vec<Rect> = traits.iter().map(Trait::bbox).collect();
            // Chaque morceau : ses traits, un mode imposé (formule entre deux `$`), et l'encre à effacer en plus
            // (les `$` eux-mêmes).
            let mut morceaux: Vec<(Vec<Trait>, Option<fano_reco::Mode>, Vec<Trait>)> = Vec::new();
            // Par blocs : une fraction, `lim` et sa borne, une matrice restent d'un seul tenant.
            for ligne in fano_ink::grouper_blocs(&boites) {
                let ts: Vec<Trait> = ligne.traits.iter().map(|&i| traits[i].clone()).collect();
                morceaux.extend(decouper_dollars(ts));
            }
            for (tous, mode_impose, a_effacer) in morceaux {
                if tous.is_empty() {
                    continue;
                }
                let ligne = fano_ink::Ligne {
                    traits: vec![],
                    bbox: tous.iter().fold(Rect::NOTHING, |r, t| r.union(t.bbox())),
                    mots: vec![],
                };
                // Une barre au-dessus des lettres (Ā, complémentaire) : elle n'est pas lue comme un « − ».
                let (barres, tous): (Vec<Trait>, Vec<Trait>) = {
                    let copie = tous.clone();
                    tous.into_iter().partition(|t| est_barre_dessus(t, &copie))
                };
                if tous.is_empty() {
                    continue;
                }
                // Un trait plat et large sous les mots les souligne : il n'est pas lu comme une lettre.
                let (traits_soulignement, ts): (Vec<Trait>, Vec<Trait>) = tous
                    .iter()
                    .cloned()
                    .partition(|t| est_soulignement(t, &ligne.bbox, &tous));
                if ts.is_empty() {
                    continue;
                }
                let souligne = !traits_soulignement.is_empty();
                // Un dessin (très haut ou isolé et grand) n'est pas lu, sauf si on le force.
                // Une ligne de plusieurs traits est de l'écriture, même très haute (intégrale, fraction) ;
                // seul un grand tracé presque seul est un dessin.
                if !forcee && ligne.bbox.height() > 140.0 && ts.len() < 3 {
                    continue;
                }
                let id = self.prochain;
                self.prochain += 1;
                journal_traits(id, &ts);
                let mode = mode_impose.unwrap_or(match self.mode_lecture {
                    crate::app::ModeLecture::Auto => fano_reco::Mode::Auto,
                    crate::app::ModeLecture::Texte => fano_reco::Mode::Texte,
                    crate::app::ModeLecture::Formule => fano_reco::Mode::Formule,
                });
                // La lecture Google (ML Kit, comme Gboard) quand elle est prête ; sinon la lecture libre.
                // Une formule demandée explicitement va au lecteur des formules.
                let google = mode != fano_reco::Mode::Formule && self.google_pret();
                let lue_par_google = google && {
                    let bb = ts.iter().fold(Rect::NOTHING, |r, t| r.union(t.bbox()));
                    let avant = self.texte_avant(page, &bb);
                    let langue = self.reglages.langue_ecriture.clone();
                    self.plateforme
                        .encre_lire(id, &langue, &ts, (bb.width(), bb.height()), &avant)
                };
                if !lue_par_google {
                    if let Some(lecteur) = &self.lecteur {
                        lecteur.lire(id, &ts, mode);
                    }
                }
                let barre = !barres.is_empty();
                let mut tous = tous;
                tous.extend(barres);
                tous.extend(a_effacer);
                self.lectures.insert(
                    id,
                    EnLecture {
                        page,
                        traits: tous,
                        forcee,
                        souligne,
                        barre,
                        lus: ts,
                    },
                );
            }
        }
    }

    /// Les lectures revenues : l'encre devient texte au propre.
    pub(crate) fn recevoir_lectures(&mut self) {
        let Some(lecteur) = &self.lecteur else { return };
        let recus: Vec<_> = lecteur.resultats.try_iter().collect();
        for (id, r) in recus {
            self.traiter_lecture(id, r);
        }
    }

    /// La lecture Google est-elle prête (choisie, et le modèle de la langue téléchargé) ?
    pub(crate) fn google_pret(&mut self) -> bool {
        if self.reglages.lecture_google != Some(true) {
            return false;
        }
        let langue = self.reglages.langue_ecriture.clone();
        self.plateforme.encre_etat(&langue) == Some(crate::plateforme::EtatModele::Pret)
    }

    /// Les lectures de Google : le premier candidat, les suivants en lectures voisines. Une ligne qui
    /// ressemble à une formule est relue en LaTeX si le modèle des formules est là.
    pub(crate) fn recevoir_lectures_google(&mut self) {
        for (id, r) in self.plateforme.encre_lues() {
            let r = r.map(|c| fano_reco::Lecture {
                texte: c.first().cloned().unwrap_or_default(),
                // ML Kit ne donne pas de confiance comparable : sa lecture est retenue (on la corrige d'un toucher).
                confiance: 1.0,
                alternatives: c.into_iter().skip(1).take(4).collect(),
                confiances: Vec::new(),
            });
            if let Ok(lecture) = &r {
                // Une formule se relit à sa structure (modèle des signes, embarqué) ; si elle doute, la
                // lecture de Google revient telle quelle.
                // Un candidat de Google qui a la forme d'une formule (« pa fo= moto » : des non-mots autour
                // d'un `=`) suffit (ADR 0014).
                let formule = self.mode_lecture == crate::app::ModeLecture::Auto
                    && std::iter::once(&lecture.texte)
                        .chain(&lecture.alternatives)
                        .any(|c| fano_reco::ressemble_formule(c) || fano_reco::aiguillage::est_maths(c));
                if formule {
                    if let (Some(lecteur), Some(l)) = (&self.lecteur, self.lectures.get(&id)) {
                        lecteur.lire_formule_sinon(id, &l.lus, lecture.clone());
                        continue;
                    }
                }
            }
            self.traiter_lecture(id, r);
        }
    }

    /// Le texte au propre juste avant `b` sur la page (il aide la lecture : contexte de la phrase).
    fn texte_avant(&self, page: usize, b: &Rect) -> String {
        let Some(p) = self.ouverte.as_ref().and_then(|o| o.lecon.pages.get(page)) else {
            return String::new();
        };
        p.elements
            .iter()
            .filter_map(|e| match e {
                Element::Texte(t) if t.pos.y < b.min.y + 4.0 => Some((t.pos.y, t.contenu.as_str())),
                _ => None,
            })
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, c)| {
                c.chars()
                    .rev()
                    .take(20)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Une lecture revenue (libre ou Google) : l'encre devient texte au propre.
    fn traiter_lecture(&mut self, id: u64, r: Result<fano_reco::Lecture, String>) {
        {
            let Some(l) = self.lectures.remove(&id) else {
                return;
            };
            match &r {
                Ok(x) => log::info!("lecture {id} lue : {:?} ({:.2}) voisines {:?}", x.texte, x.confiance, x.alternatives),
                Err(e) => log::info!("lecture {id} : erreur {e}"),
            }
            let lecture = match r {
                Ok(l) => l,
                Err(e) => {
                    if e == fano_reco::SANS_MODELE {
                        let m = t(self.langue, T::ModeleManquant).to_owned();
                        self.dire(m);
                    } else {
                        log::warn!("lecture : {e}");
                    }
                    return;
                }
            };
            let sure = lecture.confiance >= self.reglages.seuil || (l.forcee && lecture.confiance > 0.0);
            if lecture.texte.is_empty() || !sure {
                if !l.forcee {
                    let m = t(self.langue, T::PasLu).to_owned();
                    self.dire(m);
                }
                return;
            }
            let mut lecture = lecture;
            // Dans une phrase, les maths se mettent d'elles-mêmes entre `$` (lettre seule, ℝ, f(x)).
            if !lecture.texte.trim_start().starts_with('$') {
                let m = fano_reco::aiguillage::mathematiser(&lecture.texte);
                if m != lecture.texte {
                    let brut = std::mem::replace(&mut lecture.texte, m);
                    lecture.alternatives.insert(0, brut);
                }
            }
            if l.barre {
                let interieur = lecture
                    .texte
                    .trim()
                    .trim_matches('$')
                    .trim_start_matches('-')
                    .trim()
                    .to_owned();
                lecture.texte = format!("$\\overline{{{interieur}}}$");
            }
            self.remplacer_par_texte(l.page, &l.traits, lecture, l.souligne);
        }
    }

    fn remplacer_par_texte(
        &mut self,
        page: usize,
        traits: &[Trait],
        lecture: fano_reco::Lecture,
        souligne: bool,
    ) {
        let tout = traits.iter().fold(Rect::NOTHING, |r, t| r.union(t.bbox()));
        // La géométrie (taille, ligne de base) vient des lettres, pas du trait de soulignement.
        let lettres: Vec<Trait> = traits
            .iter()
            .filter(|t| !souligne || !est_soulignement(t, &tout, traits))
            .cloned()
            .collect();
        let b = lettres.iter().fold(Rect::NOTHING, |r, t| r.union(t.bbox()));
        let Some(o) = &self.ouverte else { return };
        let largeur_page = o.lecon.format.taille().0;
        // Les traits ont-ils été effacés entre-temps ? Alors on ne remet rien.
        let presents = o.lecon.pages.get(page).is_some_and(|p| {
            traits.iter().all(|t| {
                p.elements
                    .iter()
                    .any(|e| matches!(e, Element::Encre(x) if x == t))
            })
        });
        if !presents {
            return;
        }
        // Une étiquette courte écrite dans un rond : c'est le nom du nœud (graphe, arbre).
        let court = lecture.texte.chars().filter(|c| !c.is_whitespace()).count() <= 4
            && !lecture.texte.trim().contains(' ');
        if court {
            let centre = b.center();
            let rond = o.lecon.pages.get(page).and_then(|p| {
                let formes: Vec<&Forme> = p
                    .elements
                    .iter()
                    .filter_map(|e| if let Element::Forme(f) = e { Some(f) } else { None })
                    .collect();
                fano_ink::graphe::rond_contenant(&formes, centre).map(|k| formes[k].points[0])
            });
            if let Some(c) = rond {
                self.memoriser();
                if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
                    p.elements
                        .retain(|e| !matches!(e, Element::Encre(x) if traits.contains(x)));
                    let noeud = p.elements.iter_mut().find_map(|e| match e {
                        Element::Forme(f) if f.points.first() == Some(&c) => Some(f),
                        _ => None,
                    });
                    if let Some(f) = noeud {
                        f.etiquette = lecture.texte.trim().trim_matches('$').to_owned();
                    }
                }
                self.modifie();
                return;
            }
        }
        // Écrit dans une case de tableau : la case reçoit le texte.
        let centre = b.center();
        let case = o.lecon.pages.get(page).and_then(|p| {
            p.elements.iter().enumerate().find_map(|(i, e)| match e {
                Element::Tableau(tb) => tb.case(centre).map(|(l, c)| (i, l, c)),
                _ => None,
            })
        });
        if let Some((_, l, c)) = case {
            self.memoriser();
            if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
                p.elements
                    .retain(|e| !matches!(e, Element::Encre(x) if traits.contains(x)));
                // L'index a pu bouger : on retrouve le tableau par la case.
                let tableau = p.elements.iter_mut().find_map(|e| match e {
                    Element::Tableau(tb) if tb.case(centre) == Some((l, c)) => Some(tb),
                    _ => None,
                });
                if let Some(tb) = tableau {
                    let case = &mut tb.cellules[l][c];
                    if !case.is_empty() {
                        case.push(' ');
                    }
                    case.push_str(&lecture.texte);
                }
            }
            self.modifie();
            return;
        }
        let mut lecture = lecture;
        lecture.texte = fano_render::expr::corriger_definition(&lecture.texte);
        // Une étiquette d'une lettre au bout d'une branche d'arbre est un événement : une majuscule.
        if self.au_bout_d_une_branche(page, &b) {
            lecture = majuscule_d_etiquette(lecture);
        }
        let fonction = !crate::editeur::definitions(&lecture.texte).is_empty();
        let gras = self.est_gras(page, &lettres);
        // Taille : la hauteur de l'écriture ; un mot sans hampe ni jambage est plus bas.
        // Taille : la hauteur des minuscules de l'écriture (Caveat : hauteur d'x = 0,40 em), posée sur
        // la même ligne de base. Pas la hauteur de la ligne entière : hampes et jambages la gonflent.
        let (taille, base) = match fano_ink::bande_minuscules(&lettres) {
            Some((haut, base)) if base - haut > 2.0 => (
                ((base - haut) / 0.40)
                    .clamp(14.0, (b.height() / 0.85).max(14.0))
                    .min(72.0),
                base,
            ),
            _ => ((b.height() / 0.85).clamp(14.0, 72.0), b.max.y - b.height() * 0.2),
        };
        let texte = Texte {
            pos: Point::new(b.min.x, base - taille * 0.95),
            contenu: lecture.texte,
            taille,
            couleur: traits[0].couleur,
            largeur: (largeur_page - b.min.x - 24.0).max(b.width()),
            titre: 0,
            encadre: fano_core::Encadre::Aucun,
            gras,
            souligne,
            source: traits.to_vec(),
            alternatives: lecture.alternatives,
            ..Default::default()
        };
        self.memoriser();
        if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
            let premier = p
                .elements
                .iter()
                .position(|e| matches!(e, Element::Encre(x) if traits.contains(x)))
                .unwrap_or(p.elements.len());
            p.elements
                .retain(|e| !matches!(e, Element::Encre(x) if traits.contains(x)));
            let ou = premier.min(p.elements.len());
            p.elements.insert(ou, Element::Texte(texte));
        }
        self.selection.clear();
        self.modifie();
        if fonction {
            let m = t(self.langue, T::AstuceTracer).to_owned();
            self.dire(m);
        }
    }

    /// Le texte au propre qu'un trait plat, tracé juste dessous, vient souligner.
    fn texte_sous_trait(&self, page: usize, t: &Trait) -> Option<usize> {
        let tb = t.bbox();
        if tb.width() < 20.0 || tb.height() > (tb.width() * 0.18).max(6.0) || t.longueur() > tb.width() * 1.3
        {
            return None;
        }
        let p = self.ouverte.as_ref()?.lecon.pages.get(page)?;
        p.elements.iter().enumerate().find_map(|(i, e)| match e {
            Element::Texte(tx) if !tx.souligne => {
                let r = fano_render::scene::bbox(e);
                let taille = fano_render::scene::taille_titre(tx);
                let sous =
                    tb.center().y >= r.max.y - taille * 0.45 && tb.center().y <= r.max.y + taille * 0.5;
                let recouvre =
                    (tb.max.x.min(r.max.x) - tb.min.x.max(r.min.x)) >= (tb.width().min(r.width())) * 0.6;
                let large = tb.width() >= r.width() * 0.4;
                (sous && recouvre && large).then_some(i)
            }
            _ => None,
        })
    }

    /// Le texte en `b` est-il au bout d'une branche (un bout de segment ou de « < » juste à sa gauche) ?
    fn au_bout_d_une_branche(&self, page: usize, b: &Rect) -> bool {
        let Some(p) = self.ouverte.as_ref().and_then(|o| o.lecon.pages.get(page)) else {
            return false;
        };
        let h = b.height().max(20.0);
        p.elements.iter().any(|e| match e {
            Element::Forme(f)
                if matches!(
                    f.kind,
                    FormeKind::Segment | FormeKind::Polyligne | FormeKind::Fleche
                ) =>
            {
                f.points.iter().any(|q| {
                    q.x <= b.min.x + h * 0.3
                        && b.min.x - q.x <= h * 1.8
                        && (q.y - b.center().y).abs() <= h * 1.2
                })
            }
            _ => false,
        })
    }

    /// Une ligne écrite nettement plus épaisse que l'écriture habituelle de la page est du gras.
    /// Référence : l'épaisseur médiane des autres écritures de la page (encre et textes lus) ; sans
    /// autre écriture, le stylo fin par défaut.
    pub(crate) fn est_gras(&self, page: usize, traits: &[Trait]) -> bool {
        let longueur: f32 = traits.iter().map(|t| t.longueur().max(1.0)).sum();
        let poids = traits
            .iter()
            .map(|t| t.poids() * t.longueur().max(1.0))
            .sum::<f32>()
            / longueur.max(1.0);
        let mut autres: Vec<f32> = self
            .ouverte
            .as_ref()
            .and_then(|o| o.lecon.pages.get(page))
            .map(|p| {
                p.elements
                    .iter()
                    .flat_map(|e| match e {
                        Element::Encre(t) if !t.surligneur && !traits.contains(t) => vec![t.poids()],
                        Element::Texte(tx) if !tx.gras => tx.source.iter().map(Trait::poids).collect(),
                        _ => vec![],
                    })
                    .collect()
            })
            .unwrap_or_default();
        autres.sort_by(f32::total_cmp);
        let reference = if autres.len() >= 3 {
            autres[autres.len() / 2]
        } else {
            2.2 * 0.9
        };
        // 1,6 : l'appui naturel du stylet varie ; seul un trait nettement plus épais est du gras
        // (faux gras relevé sur le Pad 7 au stylet, 2026-11-08).
        poids >= reference * 1.6
    }

    /// Graphes et arbres : tailles, niveaux, arêtes accrochées au bord des nœuds.
    pub(crate) fn ajuster_graphes(&mut self, page: usize) {
        let mut change = false;
        if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
            let mut formes: Vec<&mut Forme> = p
                .elements
                .iter_mut()
                .filter_map(|e| if let Element::Forme(f) = e { Some(f) } else { None })
                .collect();
            change = fano_ink::graphe::ajuster(&mut formes);
        }
        if change {
            self.modifie();
        }
    }

    /// Un trait droit dont les deux bouts touchent des ronds de la page : une arête de graphe.
    fn arete_entre_ronds(&self, page: usize, t: &Trait) -> bool {
        let Some(p) = self.ouverte.as_ref().and_then(|o| o.lecon.pages.get(page)) else {
            return false;
        };
        let pts: Vec<Point> = t.points.iter().map(PointEncre::pos).collect();
        let Some(f) = fano_ink::reconnaitre_forme(&pts) else {
            return false;
        };
        if !matches!(f.kind, FormeKind::Segment | FormeKind::Fleche) {
            return false;
        }
        let touche = |q: Point| {
            p.elements.iter().any(|e| {
                matches!(e, Element::Forme(r) if matches!(r.kind, FormeKind::Ellipse | FormeKind::Noeud)
                    && r.points.len() >= 2
                    && r.points[0].dist(q) <= r.points[1].x.max(r.points[1].y) * 1.6 + 8.0)
            })
        };
        touche(f.points[0]) && touche(f.points[1])
    }

    /// Remplace des traits d'encre par les formes qu'ils dessinent.
    pub(crate) fn convertir_formes(&mut self, page: usize, traits: &[Trait]) {
        let rempli = self.rempli;
        self.memoriser();
        if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
            for t in traits {
                let pts: Vec<Point> = t.points.iter().map(PointEncre::pos).collect();
                let Some(r) = fano_ink::reconnaitre_forme(&pts) else {
                    continue;
                };
                if let Some(i) = p
                    .elements
                    .iter()
                    .position(|e| matches!(e, Element::Encre(x) if x == t))
                {
                    p.elements[i] = Element::Forme(Forme {
                        kind: r.kind,
                        points: r.points,
                        couleur: t.couleur,
                        epaisseur: t.epaisseur.max(1.6),
                        rempli,
                        graine: t.points.len() as u32,
                        etiquette: String::new(),
                        source: vec![t.clone()],
                    });
                }
            }
        }
        self.modifie();
        self.ajuster_graphes(page);
    }

    /// « Mettre au propre » : la sélection, ou toute l'encre en attente, ou toute l'encre de la page.
    pub(crate) fn au_propre_force(&mut self) {
        let page = if self.selection.is_empty() {
            self.page_vue
        } else {
            self.page_sel
        };
        let traits: Vec<(usize, Trait)> = match self.ouverte.as_ref().and_then(|o| o.lecon.pages.get(page)) {
            Some(p) => p
                .elements
                .iter()
                .enumerate()
                .filter(|(i, _)| self.selection.is_empty() || self.selection.contains(i))
                .filter_map(|(_, e)| match e {
                    Element::Encre(t) if !t.surligneur => Some((page, t.clone())),
                    _ => None,
                })
                .collect(),
            None => vec![],
        };
        self.attente.retain(|(p, _)| *p != page);
        self.lire_traits(traits, true);
        // Les formes de la sélection aussi.
        let sel = self.selection.clone();
        let mut formes = Vec::new();
        if let Some(p) = self.ouverte.as_ref().and_then(|o| o.lecon.pages.get(page)) {
            for &i in &sel {
                if let Some(Element::Encre(t)) = p.elements.get(i) {
                    let pts: Vec<Point> = t.points.iter().map(PointEncre::pos).collect();
                    let b = t.bbox();
                    // Seul un grand trait peut être une forme (une lettre ne l'est pas).
                    if b.width().max(b.height()) > 60.0 {
                        if let Some(r) = fano_ink::reconnaitre_forme(&pts) {
                            formes.push((t.clone(), r));
                        }
                    }
                }
            }
        }
        if !formes.is_empty() {
            self.memoriser();
            let rempli = self.rempli;
            if let Some(p) = self.pages().and_then(|v| v.get_mut(page)) {
                for (t, r) in formes {
                    if let Some(i) = p
                        .elements
                        .iter()
                        .position(|e| matches!(e, Element::Encre(x) if *x == t))
                    {
                        p.elements[i] = Element::Forme(Forme {
                            kind: r.kind,
                            points: r.points,
                            couleur: t.couleur,
                            epaisseur: t.epaisseur.max(1.6),
                            rempli,
                            graine: t.points.len() as u32,
                            etiquette: String::new(),
                            source: vec![t],
                        });
                    }
                }
            }
            self.modifie();
        }
    }
}

/// La forme d'un tracé, s'il est grand (≥ 40 pt) et seul de sa taille sur sa ligne (les autres traits
/// sont au moins deux fois plus petits) : un rond dessiné, pas la lettre O d'un mot.
pub(crate) fn forme_isolee(t: &Trait, ligne: &[Trait]) -> Option<fano_ink::FormeReconnue> {
    let b = t.bbox();
    let taille = b.width().max(b.height());
    if taille < 40.0 {
        return None;
    }
    // Les autres grandes formes nettes de la ligne (deux ronds côte à côte) ne comptent pas comme des lettres.
    let grande_forme = |o: &Trait| {
        let ob = o.bbox();
        ob.width().max(ob.height()) >= 40.0
            && fano_ink::reconnaitre_forme(&o.points.iter().map(PointEncre::pos).collect::<Vec<_>>())
                .is_some()
    };
    let autres: Vec<f32> = ligne
        .iter()
        .filter(|o| *o != t && !grande_forme(o))
        .map(|o| {
            let ob = o.bbox();
            ob.width().max(ob.height())
        })
        .collect();

    let pts: Vec<Point> = t.points.iter().map(PointEncre::pos).collect();
    let f = fano_ink::reconnaitre_forme(&pts)?;
    let ouverte = matches!(
        f.kind,
        FormeKind::Segment | FormeKind::Fleche | FormeKind::Polyligne
    );
    // Un trait ouvert qui coiffe de l'écriture (le √ et sa barre au-dessus de x) est un signe, pas un dessin.
    let coiffe = ligne.iter().any(|o| {
        o != t && {
            let c = o.bbox().center();
            c.x > b.min.x + 0.2 * b.width() && c.x < b.max.x && c.y > b.min.y && c.y < b.max.y
        }
    });
    if ouverte && coiffe && !matches!(f.kind, FormeKind::Segment) {
        return None;
    }
    if ouverte {
        // Une branche d'arbre (trait, « < ») : nettement plus longue que les lettres voisines (1,6 × la
        // médiane), et jamais un trait court (« 1 », « l », « - »).
        let mut tri = autres.clone();
        tri.sort_by(f32::total_cmp);
        let mediane = tri.get(tri.len() / 2).copied().unwrap_or(0.0);
        if taille < 60.0 || taille < mediane * 1.6 {
            return None;
        }
        return Some(f);
    }
    // Une forme fermée (rond, triangle…) : au moins deux fois plus grande que tout ce qui l'entoure.
    if autres.iter().any(|&a| a * 2.0 > taille) {
        return None;
    }
    Some(f)
}

/// Le journal de lecture (logcat sur Android, sortie d'erreur sur Linux ; il ne quitte pas l'appareil) :
/// les traits de chaque ligne envoyée à la lecture, en JSON compact, pour rejouer les erreurs signalées
/// dans les tests (`tool/essais/journal.py`).
fn journal_traits(id: u64, ts: &[Trait]) {
    let traits: Vec<String> = ts
        .iter()
        .map(|t| {
            let p: Vec<String> = t.points.iter().map(|p| format!("[{:.1},{:.1}]", p.x, p.y)).collect();
            format!("[{}]", p.join(","))
        })
        .collect();
    log::info!("lecture {id} traits [{}]", traits.join(","));
}

/// La règle des `$` écrits à la main (comme en LaTeX) : ce qui est entre deux `$` d'une ligne est une
/// formule, lue comme telle ; les `$` sont effacés à la mise au propre. Le reste de la ligne est lu
/// normalement. Un `$` ouvert sans son second : la suite attend (on écrit peut-être encore la formule).
pub(crate) fn decouper_dollars(ts: Vec<Trait>) -> Vec<(Vec<Trait>, Option<fano_reco::Mode>, Vec<Trait>)> {
    let pts: Vec<Vec<Point>> = ts
        .iter()
        .map(|t| t.points.iter().map(PointEncre::pos).collect())
        .collect();
    let d = fano_ink::dollars(&pts);
    let s = (0..pts.len()).filter(|&i| fano_ink::est_un_s(&pts[i])).count();
    if s > 0 || !d.is_empty() {
        log::info!("dollars : {} S, {} $ complets", s, d.len());
    }
    if d.is_empty() {
        return couper_ligne(ts)
            .into_iter()
            .map(|m| (m, None, Vec::new()))
            .collect();
    }
    let marques: Vec<(f32, f32)> = d
        .iter()
        .map(|&(a, b)| {
            let r = ts[a].bbox().union(ts[b].bbox());
            (r.min.x, r.max.x)
        })
        .collect();
    let dans_un_dollar = |i: usize| d.iter().any(|&(a, b)| a == i || b == i);
    let paires = marques.len() / 2;
    let mut textes: Vec<Vec<Trait>> = vec![Vec::new(); paires + 1];
    let mut formules: Vec<Vec<Trait>> = vec![Vec::new(); paires];
    for (i, t) in ts.iter().enumerate() {
        if dans_un_dollar(i) {
            continue;
        }
        let cx = t.bbox().center().x;
        let k = marques.iter().filter(|m| m.1 < cx).count();
        if k % 2 == 0 {
            textes[k / 2].push(t.clone());
        } else if k / 2 < paires {
            formules[k / 2].push(t.clone());
        }
        // Après un `$` ouvert sans second : on attend.
    }
    let mut res = Vec::new();
    for (k, t) in textes.into_iter().enumerate() {
        res.extend(couper_ligne(t).into_iter().map(|m| (m, None, Vec::new())));
        if let Some(f) = formules.get(k) {
            let effacer: Vec<Trait> = [d[2 * k], d[2 * k + 1]]
                .iter()
                .flat_map(|&(a, b)| [ts[a].clone(), ts[b].clone()])
                .collect();
            if !f.is_empty() {
                res.push((f.clone(), Some(fano_reco::Mode::Formule), effacer));
            }
        }
    }
    res.retain(|m| !m.0.is_empty());
    res
}

/// Coupe une ligne là où un grand vide sépare les traits (étiquettes d'un arbre, deux colonnes) :
/// chaque morceau est lu à part, à sa place (« XA » lu pour deux étiquettes, Pad 7, 2026-11-08).
pub(crate) fn couper_ligne(mut traits: Vec<Trait>) -> Vec<Vec<Trait>> {
    traits.sort_by(|a, b| a.bbox().min.x.total_cmp(&b.bbox().min.x));
    let hauteur = traits
        .iter()
        .fold(Rect::NOTHING, |r, t| r.union(t.bbox()))
        .height()
        .max(20.0);
    let mut res: Vec<Vec<Trait>> = Vec::new();
    let mut droite = f32::NEG_INFINITY;
    for t in traits {
        let b = t.bbox();
        if res.is_empty() || b.min.x - droite > hauteur * 1.5 {
            res.push(Vec::new());
            droite = f32::NEG_INFINITY;
        }
        droite = droite.max(b.max.x);
        res.last_mut().unwrap().push(t);
    }
    res
}

/// Une étiquette d'arbre lue « l », « ( », « 1 »… : si une lecture voisine est une majuscule seule, elle
/// gagne (les C d'Antsa lus « l » et « ( », Pad 7, 2026-11-08). Rien n'est inventé.
pub(crate) fn majuscule_d_etiquette(mut l: fano_reco::Lecture) -> fano_reco::Lecture {
    let seule_majuscule = |s: &str| {
        let mut c = s.trim().chars();
        matches!((c.next(), c.next()), (Some(x), None) if x.is_uppercase())
    };
    let texte = l.texte.trim().to_owned();
    if texte.chars().count() == 1 && !seule_majuscule(&texte) {
        if let Some(i) = l.alternatives.iter().position(|a| seule_majuscule(a)) {
            let m = l.alternatives.remove(i);
            l.alternatives
                .insert(0, std::mem::replace(&mut l.texte, m.trim().to_owned()));
        } else if let Some(c) = texte.chars().next().filter(|c| c.is_lowercase()) {
            // Sinon, la même lettre en capitale (un « c » au bout d'une branche est « C »).
            l.alternatives
                .insert(0, std::mem::replace(&mut l.texte, c.to_uppercase().collect()));
        }
    }
    l
}

/// Un petit trait plat posé au-dessus des lettres : la barre de Ā (événement contraire, complémentaire).
pub(crate) fn est_barre_dessus(t: &Trait, tous: &[Trait]) -> bool {
    let tb = t.bbox();
    let autres = tous
        .iter()
        .filter(|o| *o != t)
        .fold(Rect::NOTHING, |r, o| r.union(o.bbox()));
    if !autres.is_valid() || tb.width() < 6.0 {
        return false;
    }
    let plat = tb.height() <= (tb.width() * 0.25).max(4.0);
    let dessus = tb.center().y < autres.min.y + autres.height() * 0.15;
    let recouvre = (tb.max.x.min(autres.max.x) - tb.min.x.max(autres.min.x)) >= tb.width() * 0.5;
    plat && dessus && recouvre && t.longueur() <= tb.width() * 1.4
}

/// Un trait plat, large et à peu près droit, sous les autres traits d'une ligne : un soulignement.
pub(crate) fn est_soulignement(t: &Trait, ligne: &Rect, tous: &[Trait]) -> bool {
    let tb = t.bbox();
    let autres = tous
        .iter()
        .filter(|o| *o != t)
        .fold(Rect::NOTHING, |r, o| r.union(o.bbox()));
    if !autres.is_valid() || tb.width() < 20.0 {
        return false;
    }
    let plat = tb.height() <= (tb.width() * 0.18).max(6.0);
    let droit = t.longueur() <= tb.width() * 1.3;
    let large = tb.width() >= autres.width() * 0.35;
    let dessous = tb.center().y >= autres.min.y + autres.height() * 0.65;
    let recouvre = (tb.max.x.min(autres.max.x) - tb.min.x.max(autres.min.x)) >= tb.width() * 0.6;
    let _ = ligne;
    plat && droit && large && dessous && recouvre
}

fn rect_points(r: egui::Rect) -> [Pos2; 5] {
    [
        r.left_top(),
        r.right_top(),
        r.right_bottom(),
        r.left_bottom(),
        r.left_top(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use fano_core::Couleur;

    fn rond(cx: f32, cy: f32, r: f32) -> Trait {
        Trait {
            points: (0..=60)
                .map(|i| {
                    let a = i as f32 / 60.0 * std::f32::consts::TAU;
                    PointEncre {
                        x: cx + r * a.cos(),
                        y: cy + r * (1.0 + 0.05 * (3.0 * a).sin()) * a.sin(),
                        p: 0.5,
                        t: 0,
                    }
                })
                .collect(),
            couleur: Couleur::NOIR,
            epaisseur: 2.0,
            surligneur: false,
        }
    }

    #[test]
    fn deux_ronds_ne_font_pas_00() {
        // Relevé sur le Pad 7 (2026-11-08) : deux ronds lus « 00 ».
        let ligne = vec![rond(100.0, 100.0, 40.0), rond(300.0, 100.0, 40.0)];
        for t in &ligne {
            assert_eq!(forme_isolee(t, &ligne).map(|f| f.kind), Some(FormeKind::Ellipse));
        }
    }

    #[test]
    fn un_o_dans_un_mot_reste_une_lettre() {
        let ligne = vec![
            rond(100.0, 100.0, 9.0),
            rond(125.0, 100.0, 9.0),
            rond(150.0, 100.0, 9.0),
        ];
        assert!(
            ligne.iter().all(|t| forme_isolee(t, &ligne).is_none()),
            "trop petit : une lettre"
        );
        // « Oui » en grand : le O parmi des lettres cursives de même hauteur reste une lettre.
        let lettre = |x0: f32| Trait {
            points: (0..=40)
                .map(|i| {
                    let t = i as f32 / 40.0;
                    PointEncre {
                        x: x0 + t * 50.0,
                        y: 100.0 + 28.0 * (t * std::f32::consts::TAU * 2.0).sin(),
                        p: 0.5,
                        t: 0,
                    }
                })
                .collect(),
            couleur: Couleur::NOIR,
            epaisseur: 2.0,
            surligneur: false,
        };
        let mut titre = vec![rond(100.0, 100.0, 30.0)];
        titre.extend([lettre(150.0), lettre(210.0)]);
        assert!(
            forme_isolee(&titre[0], &titre).is_none(),
            "un grand O parmi des lettres de même taille reste une lettre"
        );
    }

    fn trait_de(pts: &[(f32, f32)]) -> Trait {
        Trait {
            points: pts
                .iter()
                .map(|&(x, y)| PointEncre { x, y, p: 0.5, t: 0 })
                .collect(),
            couleur: Couleur::NOIR,
            epaisseur: 2.0,
            surligneur: false,
        }
    }

    /// Une lettre approximative (un zigzag de la hauteur donnée).
    fn lettre(x: f32, y: f32, h: f32) -> Trait {
        trait_de(&[
            (x, y + h),
            (x + h * 0.3, y),
            (x + h * 0.6, y + h),
            (x + h * 0.8, y + h * 0.4),
        ])
    }

    #[test]
    fn arbre_de_probabilites() {
        // Relevé sur le Pad 7 (2026-11-08) : branches en « < », étiquettes A et Ā.
        let branche = trait_de(&[(400.0, 100.0), (250.0, 200.0), (400.0, 300.0)]);
        let a = lettre(430.0, 80.0, 40.0);
        let a2 = lettre(430.0, 280.0, 40.0);
        let ligne = vec![branche.clone(), a.clone(), a2.clone()];
        assert_eq!(
            forme_isolee(&branche, &ligne).map(|f| f.kind),
            Some(FormeKind::Polyligne),
            "la branche est une forme"
        );
        // Un petit « < » entre deux lettres reste le signe inférieur.
        let inf = trait_de(&[(140.0, 100.0), (110.0, 115.0), (140.0, 130.0)]);
        let math = vec![lettre(60.0, 95.0, 40.0), inf.clone(), lettre(160.0, 95.0, 40.0)];
        assert!(forme_isolee(&inf, &math).is_none());
    }

    #[test]
    fn etiquettes_eloignees_lues_a_part() {
        let morceaux = couper_ligne(vec![
            lettre(0.0, 0.0, 40.0),
            lettre(30.0, 0.0, 40.0),
            lettre(300.0, 0.0, 40.0),
        ]);
        assert_eq!(morceaux.len(), 2);
        assert_eq!(morceaux[0].len(), 2);
    }

    #[test]
    fn barre_de_a_barre() {
        let a = lettre(100.0, 100.0, 40.0);
        let barre = trait_de(&[(98.0, 88.0), (130.0, 89.0)]);
        let tous = vec![a.clone(), barre.clone()];
        assert!(est_barre_dessus(&barre, &tous));
        assert!(!est_barre_dessus(&a, &tous));
        let moins = trait_de(&[(60.0, 120.0), (85.0, 120.0)]);
        assert!(
            !est_barre_dessus(&moins, &[a, moins.clone()]),
            "un « − » à côté n'est pas une barre"
        );
    }

    #[test]
    fn etiquette_d_arbre_en_majuscule() {
        let l = |t: &str, alt: &[&str]| fano_reco::Lecture {
            texte: t.into(),
            confiance: 0.5,
            alternatives: alt.iter().map(|s| s.to_string()).collect(),
            confiances: vec![],
        };
        assert_eq!(majuscule_d_etiquette(l("(", &["C", "("])).texte, "C");
        assert_eq!(majuscule_d_etiquette(l("c", &[])).texte, "C");
        assert_eq!(
            majuscule_d_etiquette(l("A", &["4"])).texte,
            "A",
            "une majuscule reste"
        );
        assert_eq!(
            majuscule_d_etiquette(l("0,3", &[])).texte,
            "0,3",
            "une probabilité reste"
        );
    }
}
