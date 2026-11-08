//! Les icônes du rail, dessinées au trait (même main que les formes), sans police d'icônes.

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icone {
    Coffre,
    Ecrire,
    Formes,
    Gomme,
    Selection,
    Clavier,
    AuPropre,
    Diapos,
    Plus,
    Annuler,
    Retablir,
    Inserer,
    Tableau,
    Graphe,
    Image,
    Page,
    Exporter,
    Reglages,
}

pub fn dessiner(p: &Painter, r: Rect, icone: Icone, c: Color32) {
    let s = Stroke::new(r.width() * 0.085, c);
    let u = |x: f32, y: f32| -> Pos2 { r.min + vec2(x * r.width(), y * r.height()) };
    // Trait uniforme à bouts et jointures ronds, comme l'identité (une main au stylo, au trait).
    let ligne = |pts: &[Pos2]| {
        p.add(Shape::line(pts.to_vec(), s));
        for q in pts {
            p.circle_filled(*q, s.width / 2.0, c);
        }
    };
    match icone {
        Icone::Coffre => {
            // Un cahier : couverture, reliure, deux lignes.
            ligne(&[
                u(0.24, 0.16),
                u(0.78, 0.16),
                u(0.78, 0.84),
                u(0.24, 0.84),
                u(0.24, 0.16),
            ]);
            ligne(&[u(0.36, 0.16), u(0.36, 0.84)]);
            ligne(&[u(0.48, 0.38), u(0.66, 0.38)]);
            ligne(&[u(0.48, 0.52), u(0.66, 0.52)]);
        }
        Icone::Ecrire => {
            ligne(&[
                u(0.25, 0.80),
                u(0.30, 0.62),
                u(0.68, 0.24),
                u(0.80, 0.36),
                u(0.42, 0.74),
                u(0.25, 0.80),
            ]);
            ligne(&[u(0.60, 0.32), u(0.72, 0.44)]);
        }
        Icone::Formes => {
            p.circle_stroke(u(0.36, 0.38), r.width() * 0.17, s);
            ligne(&[u(0.50, 0.82), u(0.66, 0.50), u(0.84, 0.82), u(0.50, 0.82)]);
        }
        Icone::Gomme => {
            ligne(&[
                u(0.20, 0.62),
                u(0.52, 0.28),
                u(0.80, 0.52),
                u(0.50, 0.84),
                u(0.36, 0.84),
                u(0.20, 0.62),
            ]);
            ligne(&[u(0.36, 0.45), u(0.64, 0.70)]);
            ligne(&[u(0.50, 0.84), u(0.84, 0.84)]);
        }
        Icone::Selection => {
            let pts = [
                u(0.2, 0.25),
                u(0.8, 0.25),
                u(0.8, 0.75),
                u(0.2, 0.75),
                u(0.2, 0.25),
            ];
            for w in pts.windows(2) {
                p.extend(Shape::dashed_line(
                    &[w[0], w[1]],
                    s,
                    r.width() * 0.08,
                    r.width() * 0.06,
                ));
            }
        }
        Icone::Clavier => {
            p.rect_stroke(
                Rect::from_min_max(u(0.14, 0.30), u(0.86, 0.72)),
                r.width() * 0.06,
                s,
                egui::StrokeKind::Middle,
            );
            for (x, y) in [(0.28, 0.43), (0.43, 0.43), (0.57, 0.43), (0.72, 0.43)] {
                p.circle_filled(u(x, y), r.width() * 0.035, c);
            }
            ligne(&[u(0.32, 0.59), u(0.68, 0.59)]);
        }
        Icone::AuPropre => {
            // Une vague d'écriture qui devient nette, et une étoile.
            ligne(&[
                u(0.12, 0.72),
                u(0.22, 0.56),
                u(0.30, 0.72),
                u(0.40, 0.56),
                u(0.48, 0.72),
            ]);
            ligne(&[u(0.56, 0.72), u(0.86, 0.72)]);
            ligne(&[u(0.72, 0.14), u(0.72, 0.46)]);
            ligne(&[u(0.56, 0.30), u(0.88, 0.30)]);
        }
        Icone::Diapos => {
            p.rect_stroke(
                Rect::from_min_max(u(0.14, 0.24), u(0.86, 0.70)),
                r.width() * 0.04,
                s,
                egui::StrokeKind::Middle,
            );
            ligne(&[u(0.44, 0.36), u(0.60, 0.47), u(0.44, 0.58), u(0.44, 0.36)]);
            ligne(&[u(0.40, 0.84), u(0.50, 0.70), u(0.60, 0.84)]);
        }
        Icone::Plus => {
            for x in [0.28, 0.5, 0.72] {
                p.circle_filled(u(x, 0.5), r.width() * 0.06, c);
            }
        }
        Icone::Inserer => {
            p.circle_stroke(u(0.5, 0.5), r.width() * 0.34, s);
            ligne(&[u(0.5, 0.33), u(0.5, 0.67)]);
            ligne(&[u(0.33, 0.5), u(0.67, 0.5)]);
        }
        Icone::Tableau => {
            ligne(&[
                u(0.16, 0.22),
                u(0.84, 0.22),
                u(0.84, 0.78),
                u(0.16, 0.78),
                u(0.16, 0.22),
            ]);
            ligne(&[u(0.16, 0.41), u(0.84, 0.41)]);
            ligne(&[u(0.16, 0.6), u(0.84, 0.6)]);
            ligne(&[u(0.5, 0.22), u(0.5, 0.78)]);
        }
        Icone::Graphe => {
            ligne(&[u(0.16, 0.84), u(0.16, 0.14)]);
            ligne(&[u(0.16, 0.84), u(0.88, 0.84)]);
            let c: Vec<Pos2> = (0..=12)
                .map(|i| {
                    let x = i as f32 / 12.0;
                    u(0.2 + 0.64 * x, 0.78 - 0.6 * x * x)
                })
                .collect();
            ligne(&c);
        }
        Icone::Image => {
            ligne(&[
                u(0.14, 0.2),
                u(0.86, 0.2),
                u(0.86, 0.8),
                u(0.14, 0.8),
                u(0.14, 0.2),
            ]);
            ligne(&[
                u(0.2, 0.74),
                u(0.42, 0.48),
                u(0.58, 0.64),
                u(0.68, 0.54),
                u(0.8, 0.74),
            ]);
            p.circle_stroke(u(0.66, 0.36), r.width() * 0.06, s);
        }
        Icone::Page => {
            ligne(&[
                u(0.24, 0.12),
                u(0.62, 0.12),
                u(0.78, 0.28),
                u(0.78, 0.88),
                u(0.24, 0.88),
                u(0.24, 0.12),
            ]);
            ligne(&[u(0.62, 0.12), u(0.62, 0.28), u(0.78, 0.28)]);
        }
        Icone::Exporter => {
            ligne(&[u(0.5, 0.14), u(0.5, 0.62)]);
            ligne(&[u(0.32, 0.32), u(0.5, 0.14), u(0.68, 0.32)]);
            ligne(&[u(0.2, 0.56), u(0.2, 0.84), u(0.8, 0.84), u(0.8, 0.56)]);
        }
        Icone::Reglages => {
            for (y, x) in [(0.28, 0.36), (0.5, 0.64), (0.72, 0.42)] {
                ligne(&[u(0.16, y), u(0.84, y)]);
                p.circle_filled(u(x, y), r.width() * 0.09, c);
            }
        }
        Icone::Annuler | Icone::Retablir => {
            let sens = if icone == Icone::Annuler { 1.0 } else { -1.0 };
            let cx = 0.5;
            let f = |x: f32, y: f32| u(cx + (x - cx) * sens, y);
            let arc: Vec<Pos2> = (0..=12)
                .map(|i| {
                    let a = std::f32::consts::PI * (1.0 + i as f32 / 12.0);
                    f(0.52 - 0.26 * a.cos(), 0.58 + 0.22 * a.sin())
                })
                .collect();
            let mut pts = vec![f(0.70, 0.80)];
            pts.extend(arc.into_iter().rev());
            ligne(&pts);
            ligne(&[f(0.18, 0.40), f(0.26, 0.60), f(0.44, 0.52)]);
        }
    }
    let _ = pos2;
}
