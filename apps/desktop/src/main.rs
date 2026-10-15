#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod action_wheel;
mod app;
mod bubble_policy;
mod chat;
mod history;
mod hit_map;
mod hotkeys;
mod idle;
mod input;
mod motion;
mod placement;
mod reminders;
mod settings_window;
mod size;
mod timer;
mod tray;
mod updater;
mod window;

use buddy_core::{Engine, Settings};
use eframe::egui;

fn main() -> eframe::Result {
    let icon =
        eframe::icon_data::from_png_bytes(include_bytes!("../../../assets/app-icon/beni-256.png"))
            .expect("validated embedded Beni app icon");
    let options = eframe::NativeOptions {
        // Explicit test/development isolation, without changing HOME or the
        // installed app's settings. Relative overrides are rejected.
        persistence_path: data_directory(std::env::var_os("BENI_BUDDY_DATA_DIR"))?,
        viewport: egui::ViewportBuilder::default()
            .with_title("Beni Desktop Buddy")
            .with_app_id("io.witzelfitz.beni-buddy")
            .with_icon(icon)
            .with_inner_size(size::BuddySize::default().dimensions())
            .with_decorations(false)
            .with_resizable(false)
            .with_transparent(true)
            .with_has_shadow(false)
            .with_always_on_top()
            .with_active(false),
        renderer: eframe::Renderer::Glow,
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        "Beni Desktop Buddy",
        options,
        Box::new(|cc| {
            let engine = Engine::embedded()?;
            let settings: Settings = cc
                .storage
                .and_then(|storage| eframe::get_value(storage, "buddy-settings"))
                .unwrap_or_default();
            Ok(Box::new(app::BuddyApp::new(cc, engine, settings)?))
        }),
    )
}

fn data_directory(value: Option<std::ffi::OsString>) -> eframe::Result<Option<std::path::PathBuf>> {
    let Some(value) = value else { return Ok(None) };
    let path = std::path::PathBuf::from(value);
    if !path.is_absolute() {
        return Err(eframe::Error::AppCreation(
            "BENI_BUDDY_DATA_DIR benötigt einen absoluten Pfad.".into(),
        ));
    }
    Ok(Some(path))
}

#[cfg(test)]
mod startup_tests {
    #[test]
    fn only_explicit_absolute_data_directory_overrides_are_accepted() {
        assert!(super::data_directory(None).unwrap().is_none());
        assert!(super::data_directory(Some("relative".into())).is_err());
        assert!(super::data_directory(Some("".into())).is_err());
        let path = std::env::temp_dir().join("beni-isolated-settings");
        assert_eq!(
            super::data_directory(Some(path.clone().into())).unwrap(),
            Some(path)
        );
    }
}
