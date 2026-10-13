//! Desktop size presets; stored separately from animation settings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BuddySize {
    Small,
    #[default]
    Medium,
    Large,
}

impl BuddySize {
    pub const ALL: [Self; 3] = [Self::Small, Self::Medium, Self::Large];

    pub fn dimensions(self) -> eframe::egui::Vec2 {
        let [width, height] = match self {
            Self::Small => [300.0, 360.0],
            Self::Medium => [340.0, 440.0],
            Self::Large => [420.0, 560.0],
        };
        eframe::egui::vec2(width, height)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Small => "Klein",
            Self::Medium => "Mittel",
            Self::Large => "Gross",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }

    pub fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        match storage
            .and_then(|s| eframe::get_value::<String>(s, "buddy-size"))
            .as_deref()
        {
            Some("small") => Self::Small,
            Some("large") => Self::Large,
            _ => Self::Medium,
        }
    }

    pub fn save(self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "buddy-size", &self.key());
    }
}
