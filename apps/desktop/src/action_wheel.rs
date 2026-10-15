//! Geometry and presentation of the everyday actions around Beni.
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Focus,
    Reminder,
    History,
    Quiet,
    Chat,
    Settings,
}
impl Action {
    pub const ALL: [Self; 6] = [
        Self::Focus,
        Self::Reminder,
        Self::History,
        Self::Quiet,
        Self::Chat,
        Self::Settings,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Focus => "Fokus",
            Self::Reminder => "Erinnerung",
            Self::History => "Verlauf",
            Self::Quiet => "Ruhe",
            Self::Chat => "Reden",
            Self::Settings => "Einstellungen",
        }
    }
    pub fn accessible_label(self, quiet: bool) -> &'static str {
        match self {
            Self::Focus => "Fokus und Pause öffnen",
            Self::Reminder => "Erinnerungen öffnen",
            Self::History => "Meldungsverlauf öffnen",
            Self::Quiet if quiet => "Nicht stören deaktivieren",
            Self::Quiet => "Nicht stören aktivieren",
            Self::Chat => "Mit Beni reden",
            Self::Settings => "Einstellungen öffnen",
        }
    }
}

pub struct Layout {
    pub buttons: [Rect; 6],
}
impl Layout {
    pub fn new(area: Rect, avatar: Rect) -> Option<Self> {
        if !area.is_finite() || !avatar.is_finite() || area.width() < 264.0 || area.height() < 152.0
        {
            return None;
        }
        let size = egui::vec2(90.0, ((area.height() - 16.0) / 3.0).clamp(44.0, 52.0));
        let spread = ((area.height() - size.y - 2.0) * 0.5).min(104.0);
        let center = egui::pos2(
            area.center().x,
            avatar.center().y.clamp(
                area.top() + spread + size.y * 0.5,
                area.bottom() - spread - size.y * 0.5,
            ),
        );
        let radius = (area.width() - size.x) * 0.5;
        let offsets = [
            [-radius + 6.0, -spread],
            [radius - 6.0, -spread],
            [radius, 0.0],
            [radius - 6.0, spread],
            [-radius + 6.0, spread],
            [-radius, 0.0],
        ];
        Some(Self {
            buttons: offsets.map(|[x, y]| Rect::from_center_size(center + egui::vec2(x, y), size)),
        })
    }
    pub fn paint_backdrop(&self, painter: &egui::Painter) {
        painter.add(egui::Shape::closed_line(
            self.buttons.map(|r| r.center()).to_vec(),
            Stroke::new(1.5, Color32::from_rgba_unmultiplied(142, 202, 175, 65)),
        ));
    }
}

pub fn button(ui: &mut egui::Ui, rect: Rect, action: Action, quiet: bool) -> egui::Response {
    let selected = action == Action::Quiet && quiet;
    let response = ui.put(
        rect,
        egui::Button::new(
            egui::RichText::new(format!("\n{}", action.label()))
                .size(12.0)
                .color(Color32::from_rgb(239, 247, 242)),
        )
        .corner_radius(14)
        .fill(if selected {
            Color32::from_rgb(42, 88, 68)
        } else {
            Color32::from_rgb(24, 40, 44)
        })
        .stroke(Stroke::new(1.0, Color32::from_rgb(87, 129, 111))),
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            response.enabled(),
            action.accessible_label(quiet),
        )
    });
    let stroke = Stroke::new(
        1.5,
        if response.hovered() {
            Color32::WHITE
        } else {
            Color32::from_rgb(164, 223, 182)
        },
    );
    icon(
        ui.painter(),
        action,
        rect.center_top() + egui::vec2(0.0, 13.0),
        stroke,
    );
    if selected {
        ui.painter()
            .circle_filled(rect.right_top() + egui::vec2(-10.0, 9.0), 2.5, stroke.color);
    }
    response.on_hover_text(action.accessible_label(quiet))
}

fn icon(painter: &egui::Painter, action: Action, center: Pos2, stroke: Stroke) {
    let line = |a: Vec2, b: Vec2| painter.line_segment([center + a, center + b], stroke);
    match action {
        Action::Focus => {
            painter.circle_stroke(center, 6.5, stroke);
            line(egui::vec2(0.0, -4.0), Vec2::ZERO);
            line(Vec2::ZERO, egui::vec2(3.5, 2.0));
        }
        Action::Reminder => {
            painter.add(egui::Shape::line(
                vec![
                    center + egui::vec2(-6.0, 4.0),
                    center + egui::vec2(-4.5, -3.0),
                    center + egui::vec2(0.0, -6.0),
                    center + egui::vec2(4.5, -3.0),
                    center + egui::vec2(6.0, 4.0),
                    center + egui::vec2(-6.0, 4.0),
                ],
                stroke,
            ));
            painter.circle_filled(center + egui::vec2(0.0, 7.0), 1.5, stroke.color);
        }
        Action::History => {
            for y in [-5.0, 0.0, 5.0] {
                line(egui::vec2(-6.0, y), egui::vec2(6.0, y));
            }
        }
        Action::Quiet => {
            painter.circle_stroke(center, 6.5, stroke);
            line(egui::vec2(-4.0, 4.0), egui::vec2(4.0, -4.0));
        }
        Action::Chat => {
            painter.rect_stroke(
                Rect::from_center_size(center, egui::vec2(15.0, 10.0)),
                3,
                stroke,
                egui::StrokeKind::Inside,
            );
            line(egui::vec2(-4.0, 5.0), egui::vec2(-4.0, 8.0));
        }
        Action::Settings => {
            painter.circle_stroke(center, 4.0, stroke);
            for n in 0..8 {
                let angle = n as f32 * std::f32::consts::TAU / 8.0;
                let direction = egui::vec2(angle.cos(), angle.sin());
                line(direction * 5.0, direction * 7.5);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn orbits_fit_short_long_speech_and_every_buddy_size_without_overlapping_each_other() {
        for width in [284.0, 324.0, 404.0] {
            for height in [152.0, 180.0, 250.0, 440.0] {
                let area = Rect::from_min_size(egui::pos2(8.0, 123.0), egui::vec2(width, height));
                let avatar = Rect::from_center_size(area.center(), egui::vec2(90.0, 140.0));
                let layout = Layout::new(area, avatar).unwrap();
                for (index, rect) in layout.buttons.iter().enumerate() {
                    assert!(area.contains_rect(*rect));
                    assert!(rect.width() >= 44.0 && rect.height() >= 44.0);
                    for other in &layout.buttons[index + 1..] {
                        assert!(!rect.intersects(*other));
                    }
                }
            }
        }
        assert!(
            Layout::new(
                Rect::from_min_size(Pos2::ZERO, egui::vec2(200.0, 100.0)),
                Rect::NOTHING
            )
            .is_none()
        );
    }
}
