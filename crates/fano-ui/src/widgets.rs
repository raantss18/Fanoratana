//! Petits éléments d'interface : interrupteur, ligne de menu avec icône.

use egui::{Response, Sense, Ui, vec2};

use crate::icones::{self, Icone};
use crate::teinte;

/// Un interrupteur (Material 3), avec son libellé à gauche ; toute la ligne est cliquable (≥ 48 dp).
pub fn interrupteur(ui: &mut Ui, valeur: &mut bool, libelle: &str) -> Response {
    let largeur = ui.available_width().max(220.0);
    let (r, mut rep) = ui.allocate_exact_size(vec2(largeur, 48.0), Sense::click());
    if rep.clicked() {
        *valeur = !*valeur;
        rep.mark_changed();
    }
    rep.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, *valeur, libelle));
    let p = ui.painter();
    if rep.hovered() {
        p.rect_filled(r, 8.0, teinte::SURVOL);
    }
    let galley = p.layout(
        libelle.to_owned(),
        egui::FontId::proportional(16.0),
        teinte::GRAPHITE,
        r.width() - 76.0,
    );
    p.galley(
        egui::pos2(r.left() + 8.0, r.center().y - galley.size().y / 2.0),
        galley,
        teinte::GRAPHITE,
    );
    let t = ui.ctx().animate_bool_responsive(rep.id, *valeur);
    let piste = egui::Rect::from_center_size(egui::pos2(r.right() - 34.0, r.center().y), vec2(44.0, 24.0));
    let fond = if *valeur { teinte::GRAPHITE } else { teinte::FILET };
    p.rect_filled(piste, 12.0, fond);
    let cx = egui::lerp(piste.left() + 12.0..=piste.right() - 12.0, t);
    p.circle_filled(egui::pos2(cx, piste.center().y), 9.0, egui::Color32::WHITE);
    if rep.has_focus() {
        p.rect_stroke(
            r,
            8.0,
            egui::Stroke::new(2.0, teinte::GRAPHITE),
            egui::StrokeKind::Inside,
        );
    }
    rep
}

/// Une ligne de menu : icône au trait, libellé, coche si choisie.
pub fn ligne(ui: &mut Ui, icone: Option<Icone>, libelle: &str, choisie: bool) -> Response {
    let largeur = ui.available_width().max(220.0);
    let (r, rep) = ui.allocate_exact_size(vec2(largeur, 48.0), Sense::click());
    rep.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, choisie, libelle));
    let p = ui.painter();
    if choisie {
        p.rect_filled(r, 8.0, teinte::SURVOL);
    } else if rep.hovered() {
        p.rect_filled(r, 8.0, teinte::SURVOL);
    }
    let mut x = r.left() + 10.0;
    if let Some(i) = icone {
        icones::dessiner(
            p,
            egui::Rect::from_center_size(egui::pos2(x + 13.0, r.center().y), vec2(26.0, 26.0)),
            i,
            teinte::GRAPHITE,
        );
        x += 38.0;
    }
    p.text(
        egui::pos2(x, r.center().y),
        egui::Align2::LEFT_CENTER,
        libelle,
        egui::FontId::proportional(16.0),
        teinte::GRAPHITE,
    );
    if choisie {
        let c = egui::pos2(r.right() - 22.0, r.center().y);
        p.add(egui::Shape::line(
            vec![c + vec2(-7.0, 0.0), c + vec2(-2.0, 5.0), c + vec2(7.0, -6.0)],
            egui::Stroke::new(2.4, teinte::GRAPHITE),
        ));
    }
    if rep.has_focus() {
        p.rect_stroke(
            r,
            8.0,
            egui::Stroke::new(2.0, teinte::GRAPHITE),
            egui::StrokeKind::Inside,
        );
    }
    rep
}

/// Un titre de section dans un menu.
pub fn section(ui: &mut Ui, titre: &str) {
    ui.add_space(4.0);
    ui.label(egui::RichText::new(titre).size(14.0).color(teinte::DOUX));
}
