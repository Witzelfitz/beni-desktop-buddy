use eframe::egui;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Settings,
    Focus,
    Reminders,
}
impl Page {
    pub fn title(self) -> &'static str {
        match self {
            Self::Settings => "Benis Einstellungen",
            Self::Focus => "Benis Fokus & Pause",
            Self::Reminders => "Benis Erinnerungen",
        }
    }
}

pub fn id() -> egui::ViewportId {
    egui::ViewportId::from_hash_of("beni-settings-window")
}

pub fn builder() -> egui::ViewportBuilder {
    egui::ViewportBuilder::default()
        .with_title("Benis Einstellungen")
        .with_app_id("io.witzelfitz.beni-buddy.settings")
        .with_inner_size([520.0, 680.0])
        .with_min_inner_size([380.0, 360.0])
        .with_clamp_size_to_monitor_size(true)
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_mouse_passthrough(false)
        .with_window_level(egui::WindowLevel::Normal)
        .with_taskbar(true)
        .with_active(true)
}

impl Page {
    pub fn is_card(self) -> bool {
        self != Self::Settings
    }
    pub fn card_size(self) -> egui::Vec2 {
        egui::vec2(
            348.0,
            if self == Self::Reminders {
                320.0
            } else {
                332.0
            },
        )
    }
}

pub fn card_builder(page: Page, position: Option<egui::Pos2>) -> egui::ViewportBuilder {
    let builder = egui::ViewportBuilder::default()
        .with_title(page.title())
        .with_app_id("io.witzelfitz.beni-buddy.settings")
        .with_inner_size(page.card_size())
        .with_min_inner_size([260.0, 220.0])
        .with_clamp_size_to_monitor_size(true)
        .with_decorations(false)
        .with_resizable(false)
        .with_transparent(true)
        .with_has_shadow(false)
        .with_mouse_passthrough(false)
        .with_taskbar(false)
        .with_active(true);
    if let Some(position) = position {
        builder.with_position(position)
    } else {
        builder
    }
}

pub fn card_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(250, 248, 253))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgb(222, 216, 233),
        ))
        .corner_radius(20)
        .inner_margin(20)
        .shadow(egui::epaint::Shadow {
            offset: [0, 3],
            blur: 12,
            spread: 0,
            color: egui::Color32::from_black_alpha(30),
        })
}

pub fn style(ui: &mut egui::Ui) {
    let style = ui.style_mut();
    style.visuals = egui::Visuals::light();
    style.visuals.override_text_color = Some(egui::Color32::from_rgb(45, 43, 57));
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(214, 230, 224);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(39, 71, 61));
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 9.0);
}

/// Logical screen coordinates, including negative origins of secondary monitors.
pub fn card_position(
    anchor: egui::Rect,
    screens: &[egui::Rect],
    size: egui::Vec2,
) -> Option<egui::Pos2> {
    if !anchor.is_finite()
        || !anchor.is_positive()
        || !size.is_finite()
        || size.x <= 0.0
        || size.y <= 0.0
    {
        return None;
    }
    let screen = screens
        .iter()
        .filter(|s| s.is_finite() && s.width() > 16.0 && s.height() > 16.0)
        .find(|s| s.contains(anchor.center()))
        .or_else(|| {
            screens
                .iter()
                .find(|s| s.is_finite() && s.width() > 16.0 && s.height() > 16.0)
        })?
        .shrink(8.0);
    let y = (anchor.bottom() - size.y - 32.0)
        .clamp(screen.top(), (screen.bottom() - size.y).max(screen.top()));
    for x in [anchor.left() - size.x - 12.0, anchor.right() + 12.0] {
        let candidate = egui::pos2(x, y);
        if screen.contains_rect(egui::Rect::from_min_size(candidate, size)) {
            return Some(candidate);
        }
    }
    Some(egui::pos2(
        (anchor.center().x - size.x * 0.5)
            .clamp(screen.left(), (screen.right() - size.x).max(screen.left())),
        (anchor.top() - size.y - 12.0)
            .clamp(screen.top(), (screen.bottom() - size.y).max(screen.top())),
    ))
}

pub fn primary(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    ui.add_enabled_ui(enabled, |ui| {
        ui.add_sized(
            egui::vec2(ui.available_width(), 40.0),
            egui::Button::new(
                egui::RichText::new(label).color(egui::Color32::from_rgb(247, 250, 248)),
            )
            .fill(egui::Color32::from_rgb(49, 83, 74))
            .corner_radius(10),
        )
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cards_stay_on_the_buddys_screen_and_prefer_free_side_space() {
        let size = Page::Focus.card_size();
        let screens = [
            egui::Rect::from_min_size(egui::pos2(-1440.0, 24.0), egui::vec2(1440.0, 876.0)),
            egui::Rect::from_min_size(egui::pos2(0.0, 24.0), egui::vec2(1920.0, 1056.0)),
        ];
        for screen in screens {
            for origin in [
                screen.min,
                screen.max - egui::vec2(340.0, 440.0),
                screen.center() - egui::vec2(170.0, 220.0),
            ] {
                let anchor = egui::Rect::from_min_size(origin, egui::vec2(340.0, 440.0));
                let position = card_position(anchor, &screens, size).unwrap();
                assert!(screen.contains_rect(egui::Rect::from_min_size(position, size)));
            }
        }
        assert!(card_position(egui::Rect::NOTHING, &screens, size).is_none());
        assert!(card_position(screens[0], &[], size).is_none());
        assert!(card_position(screens[0], &screens, egui::vec2(f32::NAN, 20.0)).is_none());
    }
}
