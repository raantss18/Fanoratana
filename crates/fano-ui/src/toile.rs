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
                    zone.top() + ECART + self.defil.y + i as f32 * (h + ECART) * self.zoom,
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
        let zone = ui.max_rect();
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
        let total = n * (h + ECART) * self.zoom + ECART + CARTE * self.zoom;
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
        // En-tête discret : où on est, et la page.
        if let Some(o) = &self.ouverte {
            let titre = format!("{} — {}", o.fil, o.lecon.titre);
            let n = o.lecon.pages.len();
            // Le fil a son propre fond : il ne se mêle jamais au contenu qui défile dessous.
            let galley =
                painter.layout_no_wrap(titre.clone(), egui::FontId::proportional(14.0), teinte::DOUX);
            let fond =
                egui::Rect::from_min_size(zone.left_top() + vec2(8.0, 2.0), galley.size() + vec2(16.0, 8.0));
            painter.rect_filled(fond, 6.0, Color32::from_rgba_unmultiplied(0xf1, 0xf1, 0xf1, 235));
            painter.text(
                zone.left_top() + vec2(16.0, 6.0),
                egui::Align2::LEFT_TOP,
                titre,
                egui::FontId::proportional(14.0),
                teinte::DOUX,
            );
            if n > 1 {
                painter.text(
                    zone.left_bottom() + vec2(16.0, -10.0),
                    egui::Align2::LEFT_BOTTOM,
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
                _ if doigt_deplace && matches!(self.outil, Outil::Ecrire | Outil::Formes) => {
                    Some(Geste::Defiler)
                }
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
        let haut = Pos2::new(zone.center().x, zone.top() + ECART);
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
    }

    /// Envoie des traits à la lecture, ligne par ligne.
    pub(crate) fn lire_traits(&mut self, traits: Vec<(usize, Trait)>, forcee: bool) {
        let Some(lecteur) = &self.lecteur else { return };
        let mut par_page: std::collections::BTreeMap<usize, Vec<Trait>> = Default::default();
        for (p, t) in traits {
            par_page.entry(p).or_default().push(t);
        }
        for (page, traits) in par_page {
            let boites: Vec<Rect> = traits.iter().map(Trait::bbox).collect();
            for ligne in fano_ink::grouper_lignes(&boites) {
                let ts: Vec<Trait> = ligne.traits.iter().map(|&i| traits[i].clone()).collect();
                // Un dessin (très haut ou isolé et grand) n'est pas lu, sauf si on le force.
                if !forcee && ligne.bbox.height() > 140.0 {
                    continue;
                }
                let id = self.prochain;
                self.prochain += 1;
                let mode = match self.mode_lecture {
                    crate::app::ModeLecture::Auto => fano_reco::Mode::Auto,
                    crate::app::ModeLecture::Texte => fano_reco::Mode::Texte,
                    crate::app::ModeLecture::Formule => fano_reco::Mode::Formule,
                };
                lecteur.lire(id, &ts, mode);
                self.lectures.insert(
                    id,
                    EnLecture {
                        page,
                        traits: ts,
                        forcee,
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
            let Some(l) = self.lectures.remove(&id) else {
                continue;
            };
            let lecture = match r {
                Ok(l) => l,
                Err(e) => {
                    if e == fano_reco::SANS_MODELE {
                        let m = t(self.langue, T::ModeleManquant).to_owned();
                        self.dire(m);
                    } else {
                        log::warn!("lecture : {e}");
                    }
                    continue;
                }
            };
            let sure = lecture.confiance >= self.reglages.seuil || (l.forcee && lecture.confiance > 0.0);
            if lecture.texte.is_empty() || !sure {
                if !l.forcee {
                    let m = t(self.langue, T::PasLu).to_owned();
                    self.dire(m);
                }
                continue;
            }
            self.remplacer_par_texte(l.page, &l.traits, lecture);
        }
    }

    fn remplacer_par_texte(&mut self, page: usize, traits: &[Trait], lecture: fano_reco::Lecture) {
        let b = traits.iter().fold(Rect::NOTHING, |r, t| r.union(t.bbox()));
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
        let fonction = !crate::editeur::definitions(&lecture.texte).is_empty();
        let gras = self.est_gras(page, traits);
        // Taille : la hauteur de l'écriture ; un mot sans hampe ni jambage est plus bas.
        let taille = (b.height() / 0.85).clamp(14.0, 72.0);
        let texte = Texte {
            pos: Point::new(b.min.x, b.min.y - taille * 0.22),
            contenu: lecture.texte,
            taille,
            couleur: traits[0].couleur,
            largeur: (largeur_page - b.min.x - 24.0).max(b.width()),
            titre: 0,
            encadre: fano_core::Encadre::Aucun,
            gras,
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
        poids >= reference * 1.45
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

fn rect_points(r: egui::Rect) -> [Pos2; 5] {
    [
        r.left_top(),
        r.right_top(),
        r.right_bottom(),
        r.left_bottom(),
        r.left_top(),
    ]
}
