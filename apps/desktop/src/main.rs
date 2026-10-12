#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod app;
mod placement;
mod tray;
mod window;

use buddy_core::{Engine, Settings};
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Beni Desktop Buddy")
            .with_app_id("io.witzelfitz.beni-buddy")
            .with_inner_size([340.0, 440.0])
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
