//! Geometry and presentation of the everyday actions around Beni.
use eframe::egui::{self, Color32, Rect, Stroke};

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

pub const FILL: Color32 = Color32::from_rgb(43, 49, 61);
pub const BORDER: Color32 = Color32::from_rgb(111, 164, 150);

pub struct Layout {
    pub buttons: [Rect; 6],
    pub avatar: Rect,
}
impl Layout {
    pub fn new(area: Rect, avatar: Rect) -> Option<Self> {
        if !area.is_finite()
            || !avatar.is_finite()
            || !avatar.is_positive()
            || area.width() < 264.0
            || area.height() < 200.0
        {
            return None;
        }
        let size = egui::vec2((area.width() * 0.27).clamp(82.0, 96.0), 44.0);
        let center = area.center() - egui::vec2(0.0, 18.0);
        let spread = ((area.height() - 84.0) * 0.5).min(96.0);
        let radius = (area.width() - size.x) * 0.5;
        let offsets = [
            [-radius + 4.0, -spread],
            [radius - 4.0, -spread],
            [-radius, 0.0],
            [radius, 0.0],
            [-radius + 4.0, spread],
            [radius - 4.0, spread],
        ];
        let space = egui::vec2(area.width() - 2.0 * size.x - 28.0, area.height() - 48.0);
        let scale = (space.x / avatar.width())
            .min(space.y / avatar.height())
            .min(1.0);
        Some(Self {
            buttons: offsets.map(|[x, y]| Rect::from_center_size(center + egui::vec2(x, y), size)),
            avatar: Rect::from_center_size(center, avatar.size() * scale),
        })
    }
}

pub fn button(ui: &mut egui::Ui, rect: Rect, action: Action, quiet: bool) -> egui::Response {
    let selected = action == Action::Quiet && quiet;
    let response = ui.put(
        rect,
        egui::Button::new(
            egui::RichText::new(action.label())
                .size(11.5)
                .color(Color32::from_rgb(244, 246, 248)),
        )
        .corner_radius(12)
        .fill(if selected {
            Color32::from_rgb(49, 83, 74)
        } else {
            FILL
        })
        .stroke(Stroke::new(1.0, BORDER)),
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            response.enabled(),
            action.accessible_label(quiet),
        )
    });
    if selected {
        ui.painter()
            .circle_filled(rect.right_top() + egui::vec2(-8.0, 8.0), 2.0, BORDER);
    }
    response.on_hover_text(action.accessible_label(quiet))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_button_clears_the_complete_avatar_and_the_other_buttons() {
        for width in [284.0, 324.0, 404.0] {
            for height in [200.0, 250.0, 440.0] {
                for avatar_size in [
                    egui::vec2(90.0, 140.0),
                    egui::vec2(250.0, 230.0),
                    egui::vec2(160.0, 300.0),
                ] {
                    let area =
                        Rect::from_min_size(egui::pos2(8.0, 123.0), egui::vec2(width, height));
                    let avatar = Rect::from_center_size(area.center(), avatar_size);
                    let layout = Layout::new(area, avatar).unwrap();
                    assert!(area.contains_rect(layout.avatar));
                    for (index, rect) in layout.buttons.iter().enumerate() {
                        assert!(area.contains_rect(*rect));
                        assert!(rect.width() >= 44.0 && rect.height() >= 44.0);
                        assert!(!rect.expand(8.0).intersects(layout.avatar));
                        for other in &layout.buttons[index + 1..] {
                            assert!(!rect.intersects(*other));
                        }
                    }
                    assert!(layout.buttons[2].center().x < layout.avatar.left());
                    assert!(layout.buttons[3].center().x > layout.avatar.right());
                }
            }
        }
    }
    #[test]
    fn constrained_or_invalid_geometry_uses_the_accessible_list() {
        for size in [
            egui::vec2(200.0, 280.0),
            egui::vec2(324.0, 152.0),
            egui::vec2(f32::NAN, 440.0),
        ] {
            let area = Rect::from_min_size(egui::Pos2::ZERO, size);
            assert!(Layout::new(area, area).is_none());
        }
        assert!(Layout::new(Rect::EVERYTHING, Rect::NOTHING).is_none());
    }
}
