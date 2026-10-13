use eframe::egui;

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
