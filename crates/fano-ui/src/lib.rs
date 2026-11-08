//! L'interface de Fanoratana : la même sur Linux et sur Android (egui).
//!
//! L'écran est à la feuille. Les outils vivent dans la marge rouge du cahier, à gauche ; le coffre
//! (Mes cours) s'ouvre par-dessus. On écrit au doigt, au stylet ou à la souris ; on tape au
//! clavier ; après une pause, l'écriture se met au propre (docs/design.md, ADR 0005).

mod app;
mod coffre_ui;
mod coins;
mod editeur;
mod google;
pub mod icones;
mod maj;
pub mod peinture;
pub mod plateforme;
pub mod reglages;
pub mod textes;
mod toile;
mod widgets;

pub use app::Fanoratana;
pub use plateforme::{EtatModele, Plateforme, Pointeur};

use egui::{FontData, FontDefinitions, FontFamily};

/// Charge les polices (Nunito pour l'interface, Caveat et Noto Sans Math pour la feuille).
pub fn installer_polices(ctx: &egui::Context) {
    let mut f = FontDefinitions::default();
    f.font_data.insert(
        "nunito".into(),
        FontData::from_static(fano_render::polices::NUNITO).into(),
    );
    f.font_data.insert(
        "caveat".into(),
        FontData::from_static(fano_render::polices::CAVEAT).into(),
    );
    f.font_data.insert(
        "noto-math".into(),
        FontData::from_static(fano_render::polices::SYMBOLES).into(),
    );
    f.families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "nunito".into());
    f.families
        .entry(FontFamily::Proportional)
        .or_default()
        .push("noto-math".into());
    f.families.insert(
        FontFamily::Name(peinture::MANUSCRIT.into()),
        vec!["caveat".into(), "noto-math".into()],
    );
    f.families.insert(
        FontFamily::Name(peinture::SYMBOLES.into()),
        vec!["noto-math".into(), "caveat".into()],
    );
    ctx.set_fonts(f);
}

/// Les couleurs de l'interface : monochrome, noir sur blanc (docs/design.md).
pub mod teinte {
    use egui::Color32;
    /// Le noir du trait : texte, icônes, outil actif.
    pub const GRAPHITE: Color32 = Color32::BLACK;
    /// Gris des textes secondaires (contraste 6,9:1 sur blanc).
    pub const DOUX: Color32 = Color32::from_rgb(0x5c, 0x5c, 0x5c);
    pub const PAPIER: Color32 = Color32::WHITE;
    /// Le bureau autour des feuilles.
    pub const BUREAU: Color32 = Color32::from_rgb(0xf1, 0xf1, 0xf1);
    /// Le filet qui sépare le rail de la feuille.
    pub const FILET: Color32 = Color32::from_rgb(0xd6, 0xd6, 0xd6);
    /// Survol.
    pub const SURVOL: Color32 = Color32::from_rgb(0xee, 0xee, 0xee);
    /// L'accent est le noir lui-même.
    pub const ENCRE: Color32 = Color32::BLACK;
    /// Alerte (supprimer) : le seul ton, réservé au danger.
    pub const DANGER: Color32 = Color32::from_rgb(0xb3, 0x26, 0x1e);
    pub const LIGNE: Color32 = Color32::from_rgb(0xe0, 0xe0, 0xe0);
}

/// Le thème clair « cahier ».
pub fn appliquer_theme(ctx: &egui::Context) {
    use egui::{FontId, TextStyle};
    ctx.set_visuals(egui::Visuals::light());
    ctx.all_styles_mut(|s| {
        s.visuals.panel_fill = teinte::PAPIER;
        s.visuals.window_fill = teinte::PAPIER;
        s.visuals.extreme_bg_color = egui::Color32::WHITE;
        s.visuals.override_text_color = Some(teinte::GRAPHITE);
        s.visuals.selection.bg_fill = teinte::SURVOL;
        s.visuals.selection.stroke.color = teinte::ENCRE;
        s.visuals.hyperlink_color = teinte::ENCRE;
        s.visuals.widgets.hovered.bg_stroke.color = teinte::ENCRE;
        s.visuals.widgets.inactive.weak_bg_fill = teinte::SURVOL;
        s.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, teinte::FILET);
        s.visuals.widgets.hovered.weak_bg_fill = teinte::LIGNE;
        s.visuals.widgets.active.weak_bg_fill = teinte::FILET;
        s.visuals.window_stroke = egui::Stroke::new(1.0, teinte::FILET);
        s.visuals.window_corner_radius = egui::CornerRadius::same(10);
        s.visuals.window_shadow = egui::Shadow {
            offset: [0, 6],
            blur: 24,
            spread: 0,
            color: egui::Color32::from_black_alpha(28),
        };
        s.spacing.button_padding = egui::vec2(12.0, 8.0);
        s.spacing.interact_size.y = 40.0;
        s.spacing.item_spacing = egui::vec2(10.0, 8.0);
        s.text_styles.insert(TextStyle::Body, FontId::proportional(17.0));
        s.text_styles
            .insert(TextStyle::Button, FontId::proportional(17.0));
        s.text_styles
            .insert(TextStyle::Heading, FontId::proportional(24.0));
        s.text_styles.insert(TextStyle::Small, FontId::proportional(14.0));
    });
}
