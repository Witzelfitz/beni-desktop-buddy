use buddy_core::Engine;
use eframe::egui;
use std::sync::mpsc::SyncSender;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Show,
    Recover,
    Hide,
    Pause,
    DoNotDisturb,
    Settings,
    History,
    CheckUpdates,
    Quit,
}
impl Action {
    fn from_id(id: &str) -> Option<Self> {
        match id {
            "beni-show" => Some(Self::Show),
            "beni-recover" => Some(Self::Recover),
            "beni-hide" => Some(Self::Hide),
            "beni-pause" => Some(Self::Pause),
            "beni-do-not-disturb" => Some(Self::DoNotDisturb),
            "beni-settings" => Some(Self::Settings),
            "beni-history" => Some(Self::History),
            "beni-check-updates" => Some(Self::CheckUpdates),
            "beni-quit" => Some(Self::Quit),
            _ => None,
        }
    }
}
pub struct Tray {
    _icon: TrayIcon,
    pause: CheckMenuItem,
    do_not_disturb: CheckMenuItem,
    updates: MenuItem,
}
impl Tray {
    pub fn new(
        ctx: egui::Context,
        sender: SyncSender<Action>,
        engine: &Engine,
        paused: bool,
        quiet: bool,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let menu = Menu::new();
        let show = MenuItem::with_id("beni-show", "Beni anzeigen", true, None);
        let recover = MenuItem::with_id("beni-recover", "Beni zurückholen", true, None);
        let hide = MenuItem::with_id("beni-hide", "Beni ausblenden", true, None);
        let pause = CheckMenuItem::with_id("beni-pause", "Animation pausieren", true, paused, None);
        let do_not_disturb =
            CheckMenuItem::with_id("beni-do-not-disturb", "Nicht stören", true, quiet, None);
        let settings = MenuItem::with_id("beni-settings", "Einstellungen", true, None);
        let history = MenuItem::with_id("beni-history", "Meldungsverlauf", true, None);
        let updates = MenuItem::with_id("beni-check-updates", "Nach Updates suchen …", true, None);
        let quit = MenuItem::with_id("beni-quit", "Beenden", true, None);
        menu.append_items(&[
            &show,
            &recover,
            &hide,
            &PredefinedMenuItem::separator(),
            &pause,
            &do_not_disturb,
            &settings,
            &history,
            &updates,
            &PredefinedMenuItem::separator(),
            &quit,
        ])?;
        let icon = avatar_icon(engine)?;
        let builder = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .with_tooltip("Mini-Beni Desktop Buddy")
            .with_menu_on_left_click(true);
        #[cfg(target_os = "macos")]
        let builder = builder.with_title("Beni");
        let icon = builder.build()?;
        // Wake eframe even while hidden/minimized; polling the render path would lose events.
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            if let Some(action) = Action::from_id(event.id.as_ref()) {
                let _ = sender.try_send(action);
                ctx.request_repaint();
            }
        }));
        Ok(Self {
            _icon: icon,
            pause,
            do_not_disturb,
            updates,
        })
    }
    pub fn set_paused(&self, paused: bool) {
        self.pause.set_checked(paused);
    }
    pub fn set_do_not_disturb(&self, quiet: bool) {
        self.do_not_disturb.set_checked(quiet);
    }
    pub fn set_updates_enabled(&self, enabled: bool) {
        self.updates.set_enabled(enabled);
    }
}
fn avatar_icon(engine: &Engine) -> Result<Icon, Box<dyn std::error::Error + Send + Sync>> {
    let atlas = eframe::icon_data::from_png_bytes(buddy_core::ATLAS_PNG)?;
    let [x, y, w, h, _, _] = engine.atlas.frames["front"];
    // Reuse Beni's original head, sampled directly from the sprite atlas at runtime.
    let head = h.min(210);
    let size = 64_usize;
    let mut rgba = vec![0; size * size * 4];
    for row in 0..size {
        for col in 0..size {
            let sx = x as usize + col * w as usize / size;
            let sy = y as usize + row * head as usize / size;
            let src = (sy * atlas.width as usize + sx) * 4;
            let dst = (row * size + col) * 4;
            rgba[dst..dst + 4].copy_from_slice(&atlas.rgba[src..src + 4]);
        }
    }
    Ok(Icon::from_rgba(rgba, size as u32, size as u32)?)
}
