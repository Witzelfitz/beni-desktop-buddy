#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod app;
mod history;
mod hit_map;
mod hotkeys;
mod idle;
mod input;
mod motion;
mod placement;
mod settings_window;
mod size;
mod tray;
mod window;

use buddy_core::{Engine, Settings};
use eframe::egui;

fn main() -> eframe::Result {
    let icon =
        eframe::icon_data::from_png_bytes(include_bytes!("../../../assets/app-icon/beni-256.png"))
            .expect("validated embedded Beni app icon");
    let options = eframe::NativeOptions {
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
