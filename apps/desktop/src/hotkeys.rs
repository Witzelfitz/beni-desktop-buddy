//! Opt-in native shortcuts. The process-wide event callback is installed once;
//! registrations and routing can be changed without replacing that callback.
use crate::tray::Action;
use eframe::egui;
use global_hotkey::{
    GlobalHotKeyEvent, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use std::{
    collections::HashSet,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, sync_channel},
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Config {
    enabled: bool,
    show: String,
    pause: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            show: "Ctrl+Alt+B".into(),
            pause: "Ctrl+Alt+P".into(),
        }
    }
}
impl Config {
    fn keys(&self) -> Result<[HotKey; 2], String> {
        fn parse(text: &str) -> Result<HotKey, String> {
            if text.len() > 80 || text.chars().any(char::is_control) {
                return Err("Die Tastenkombination ist ungültig.".into());
            }
            let key = text
                .trim()
                .parse::<HotKey>()
                .map_err(|_| "Ungültige Kombination. Beispiel: Ctrl+Alt+B.".to_string())?;
            let name = key.key.to_string();
            let function = name
                .strip_prefix('F')
                .and_then(|n| n.parse::<u8>().ok())
                .is_some_and(|n| (1..=24).contains(&n));
            let ordinary = (name.starts_with("Key") && name.len() == 4)
                || (name.starts_with("Digit") && name.len() == 6)
                || matches!(
                    key.key,
                    Code::Space
                        | Code::Enter
                        | Code::Tab
                        | Code::Escape
                        | Code::Backspace
                        | Code::Delete
                        | Code::Insert
                        | Code::Home
                        | Code::End
                        | Code::PageUp
                        | Code::PageDown
                        | Code::ArrowUp
                        | Code::ArrowDown
                        | Code::ArrowLeft
                        | Code::ArrowRight
                        | Code::Minus
                        | Code::Equal
                        | Code::Comma
                        | Code::Period
                        | Code::Slash
                        | Code::Semicolon
                        | Code::Quote
                        | Code::Backslash
                        | Code::BracketLeft
                        | Code::BracketRight
                        | Code::Backquote
                );
            // Media keys can require a broad event tap on macOS; do not register them.
            if !ordinary && !function {
                return Err("Bitte eine normale Taste oder Funktionstaste verwenden.".into());
            }
            if !function
                && !key
                    .mods
                    .intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER)
            {
                return Err(
                    "Buchstaben und Ziffern benötigen Ctrl, Alt/Option oder Cmd/Super.".into(),
                );
            }
            Ok(key)
        }
        let keys = [parse(&self.show)?, parse(&self.pause)?];
        if keys[0].id() == keys[1].id() {
            return Err("Bitte zwei unterschiedliche Kombinationen wählen.".into());
        }
        Ok(keys)
    }
}

trait Backend {
    fn register(&mut self, key: HotKey) -> Result<(), String>;
    fn unregister(&mut self, key: HotKey) -> Result<(), String>;
}
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos::Native;
#[cfg(not(target_os = "macos"))]
struct Native(global_hotkey::GlobalHotKeyManager);
#[cfg(not(target_os = "macos"))]
impl Native {
    fn new() -> Result<Self, String> {
        global_hotkey::GlobalHotKeyManager::new()
            .map(Self)
            .map_err(|e| e.to_string())
    }
}
#[cfg(not(target_os = "macos"))]
impl Backend for Native {
    fn register(&mut self, key: HotKey) -> Result<(), String> {
        self.0.register(key).map_err(|e| e.to_string())
    }
    fn unregister(&mut self, key: HotKey) -> Result<(), String> {
        self.0.unregister(key).map_err(|e| e.to_string())
    }
}
#[derive(Debug)]
struct ChangeError {
    message: String,
    preserved: bool,
}
struct Registry<B: Backend> {
    backend: B,
    keys: Vec<HotKey>,
}
impl<B: Backend> Registry<B> {
    fn replace(&mut self, next: [HotKey; 2]) -> Result<(), ChangeError> {
        let previous = self.keys.clone();
        let mut added = Vec::new();
        // Register candidates before releasing previous bindings. Conflicts cannot
        // silently remove an already working shortcut; swaps reuse owned keys.
        for key in next {
            if previous.contains(&key) {
                continue;
            }
            if self.backend.register(key).is_err() {
                let mut preserved = true;
                for key in added.into_iter().rev() {
                    if self.backend.unregister(key).is_ok() {
                        self.keys.retain(|k| *k != key);
                    } else {
                        preserved = false;
                    }
                }
                return Err(ChangeError {
                    message: if preserved {
                        format!(
                            "{key} konnte nicht registriert werden. Bitte eine andere Kombination wählen."
                        )
                    } else {
                        "Die Registrierung und anschliessende Freigabe sind fehlgeschlagen. Bitte Beni neu starten.".into()
                    },
                    preserved,
                });
            }
            self.keys.push(key);
            added.push(key);
        }
        for key in previous {
            if !next.contains(&key) {
                if self.backend.unregister(key).is_err() {
                    return Err(ChangeError { message: "Alte Hotkeys konnten nicht freigegeben werden. Hotkeys wurden deaktiviert. Bitte Beni neu starten.".into(), preserved: false });
                }
                self.keys.retain(|k| *k != key);
            }
        }
        Ok(())
    }
}
impl<B: Backend> Drop for Registry<B> {
    fn drop(&mut self) {
        for key in self.keys.drain(..) {
            let _ = self.backend.unregister(key);
        }
    }
}

#[derive(Default)]
struct Routing {
    generation: u64,
    keys: Option<[HotKey; 2]>,
    held: HashSet<u32>,
}
impl Routing {
    fn set(&mut self, keys: Option<[HotKey; 2]>) {
        self.generation = self.generation.wrapping_add(1);
        self.keys = keys;
        self.held.clear();
    }
    fn event(&mut self, event: GlobalHotKeyEvent) -> Option<(u64, Action)> {
        let keys = self.keys?;
        let action = if event.id == keys[0].id() {
            Action::Recover
        } else if event.id == keys[1].id() {
            Action::Pause
        } else {
            return None;
        };
        match event.state {
            HotKeyState::Released => {
                self.held.remove(&event.id);
                None
            }
            HotKeyState::Pressed if self.held.insert(event.id) => Some((self.generation, action)),
            _ => None,
        }
    }
}

pub struct Hotkeys {
    config: Config,
    draft: Config,
    request: Option<Config>,
    registry: Option<Registry<Native>>,
    routing: Arc<Mutex<Routing>>,
    receiver: Option<Receiver<(u64, Action)>>,
    supported: bool,
    pub(crate) error: Option<String>,
}
impl Hotkeys {
    pub fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        let config = storage
            .and_then(|s| eframe::get_value::<(bool, String, String)>(s, "global-hotkeys"))
            .map(|(enabled, show, pause)| Config {
                enabled,
                show,
                pause,
            })
            .filter(|c| c.keys().is_ok())
            .unwrap_or_default();
        Self {
            draft: config.clone(),
            request: Some(config.clone()),
            config,
            registry: None,
            routing: Arc::new(Mutex::new(Routing::default())),
            receiver: None,
            supported: true,
            error: None,
        }
    }
    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(
            storage,
            "global-hotkeys",
            &(self.config.enabled, &self.config.show, &self.config.pause),
        );
    }
    fn disable(&mut self) {
        self.routing.lock().unwrap().set(None);
        self.registry = None;
        self.config.enabled = false;
        self.request = None;
    }
    fn request(&mut self, enabled: bool) {
        let mut next = self.draft.clone();
        next.enabled = enabled;
        match next.keys() {
            Ok(_) => {
                next.show = next.show.trim().into();
                next.pause = next.pause.trim().into();
                self.request = Some(next);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Tastenkürzel", |ui| {
            if !self.supported { ui.label("Globale Hotkeys sind unter Wayland noch nicht verfügbar. Bitte das Tray-Menü verwenden."); }
            ui.add_enabled_ui(self.supported, |ui| {
                let mut enabled = self.config.enabled;
                if ui.checkbox(&mut enabled, "Globale Hotkeys aktivieren").changed() {
                    if enabled { self.request(true); } else { self.disable(); self.error = None; }
                }
                for (label, text) in [("Beni anzeigen und zurückholen", &mut self.draft.show), ("Animation pausieren/fortsetzen", &mut self.draft.pause)] {
                    ui.label(label);
                    let response = ui.add(egui::TextEdit::singleline(text).char_limit(80).desired_width(ui.available_width()));
                    ui.ctx().accesskit_node_builder(response.id, |node| node.set_label(format!("Hotkey: {label}")));
                }
                ui.small("Beispiele: Ctrl+Alt+B, Cmd+Shift+B oder F8. Super = Windows-/Cmd-Taste. Buchstaben verwenden US-Tastenpositionen (Y/Z bei QWERTZ vertauscht).");
                if ui.button("Hotkeys übernehmen").clicked() { self.request(self.config.enabled); }
            });
            if let Some(error) = &self.error { ui.colored_label(egui::Color32::from_rgb(255, 180, 150), error); }
            if self.config.enabled { ui.small(format!("Aktiv: {} · {}", self.config.show, self.config.pause)); }
            else { ui.small("Hotkeys sind ausgeschaltet."); }
        });
    }
    pub fn sync(&mut self, ctx: &egui::Context, window: &winit::window::Window) {
        self.supported = supported(window);
        if self.receiver.is_none() && self.supported {
            let (sender, receiver) = sync_channel(16);
            let routing = self.routing.clone();
            let ctx = ctx.clone();
            // global-hotkey 0.8 uses OnceCell: never replace this process-wide callback.
            GlobalHotKeyEvent::set_event_handler(Some(move |event| {
                let action = routing.lock().ok().and_then(|mut r| r.event(event));
                if let Some(action) = action {
                    let _ = sender.try_send(action);
                    ctx.request_repaint();
                }
            }));
            self.receiver = Some(receiver);
        }
        let Some(next) = self.request.take() else {
            return;
        };
        if !next.enabled {
            self.disable();
            self.config = next;
            return;
        }
        if !self.supported {
            self.disable();
            self.error = Some("Globale Hotkeys sind unter Wayland noch nicht verfügbar.".into());
            return;
        }
        let Ok(keys) = next.keys() else {
            self.error = Some("Ungültige Hotkeys.".into());
            return;
        };
        if self.registry.is_none() {
            match Native::new() {
                Ok(backend) => {
                    self.registry = Some(Registry {
                        backend,
                        keys: Vec::new(),
                    })
                }
                Err(_) => {
                    self.disable();
                    self.error = Some("Globale Hotkeys sind momentan nicht verfügbar.".into());
                    return;
                }
            }
        }
        match self.registry.as_mut().unwrap().replace(keys) {
            Ok(()) => {
                self.routing.lock().unwrap().set(Some(keys));
                self.config = next;
                self.error = None;
            }
            Err(error) => {
                if !error.preserved || !self.config.enabled {
                    self.disable();
                }
                self.error = Some(error.message);
            }
        }
    }
    pub fn pop(&mut self) -> Option<Action> {
        let receiver = self.receiver.as_ref()?;
        while let Ok((generation, action)) = receiver.try_recv() {
            let routing = self.routing.lock().ok()?;
            if self.config.enabled && routing.keys.is_some() && generation == routing.generation {
                return Some(action);
            }
        }
        None
    }
}
impl Drop for Hotkeys {
    fn drop(&mut self) {
        self.disable();
    }
}

#[cfg(not(target_os = "linux"))]
fn supported(_: &winit::window::Window) -> bool {
    true
}
#[cfg(target_os = "linux")]
fn supported(window: &winit::window::Window) -> bool {
    use winit::raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
    !std::env::var("XDG_SESSION_TYPE").is_ok_and(|session| session.eq_ignore_ascii_case("wayland"))
        && window
            .display_handle()
            .is_ok_and(|handle| matches!(handle.as_raw(), RawDisplayHandle::Xlib(_)))
}

#[cfg(test)]
impl Hotkeys {
    pub(crate) fn test_queue(&mut self, action: Action) {
        self.config.enabled = true;
        self.routing
            .lock()
            .unwrap()
            .set(Some(self.config.keys().unwrap()));
        let generation = self.routing.lock().unwrap().generation;
        let (sender, receiver) = sync_channel(16);
        sender.try_send((generation, action)).unwrap();
        self.receiver = Some(receiver);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    #[derive(Default)]
    struct Fake {
        owned: Rc<RefCell<HashSet<u32>>>,
        blocked: Option<u32>,
        unregister_failure: Option<u32>,
    }
    impl Backend for Fake {
        fn register(&mut self, key: HotKey) -> Result<(), String> {
            if self.blocked == Some(key.id()) {
                return Err("occupied".into());
            }
            assert!(
                self.owned.borrow_mut().insert(key.id()),
                "duplicate native registration"
            );
            Ok(())
        }
        fn unregister(&mut self, key: HotKey) -> Result<(), String> {
            if self.unregister_failure == Some(key.id()) {
                self.unregister_failure = None;
                return Err("unregister failed".into());
            }
            assert!(self.owned.borrow_mut().remove(&key.id()));
            Ok(())
        }
    }
    fn keys(show: &str, pause: &str) -> [HotKey; 2] {
        Config {
            show: show.into(),
            pause: pause.into(),
            ..Default::default()
        }
        .keys()
        .unwrap()
    }
    #[test]
    fn editor_has_named_fields_and_disables_controls_when_unsupported() {
        for supported in [true, false] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            ctx.all_styles_mut(|style| style.animation_time = 0.0);
            let mut hotkeys = Hotkeys::load(None);
            hotkeys.supported = supported;
            let mut input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(340.0, 560.0),
                )),
                ..Default::default()
            };
            let output = ctx.run_ui(input.clone(), |ui| hotkeys.ui(ui));
            let target = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, n)| n.label() == Some("Tastenkürzel"))
                .unwrap()
                .0;
            output.drop_without_applying_deltas();
            input.events.push(egui::Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Click,
                    target_tree: egui::accesskit::TreeId::ROOT,
                    target_node: target,
                    data: None,
                },
            ));
            let output = ctx.run_ui(input, |ui| hotkeys.ui(ui));
            let nodes = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .clone();
            output.drop_without_applying_deltas();
            for label in [
                "Hotkey: Beni anzeigen und zurückholen",
                "Hotkey: Animation pausieren/fortsetzen",
            ] {
                let node = &nodes
                    .iter()
                    .find(|(_, node)| node.label() == Some(label))
                    .unwrap()
                    .1;
                assert_eq!(node.role(), egui::accesskit::Role::TextInput);
                assert_eq!(node.is_disabled(), !supported);
            }
            let node = &nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Globale Hotkeys aktivieren"))
                .unwrap()
                .1;
            assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::False));
            assert_eq!(node.is_disabled(), !supported);
        }
    }
    #[test]
    fn validates_shortcuts_and_rejects_duplicates_plain_text_media_and_bad_input() {
        for (show, pause) in [
            ("Ctrl+Alt+B", "Ctrl+Alt+P"),
            ("Cmd+Shift+B", "Super+Shift+P"),
            ("F8", "F9"),
            (" Ctrl + Alt + B ", "Ctrl+Space"),
        ] {
            assert!(
                Config {
                    show: show.into(),
                    pause: pause.into(),
                    ..Default::default()
                }
                .keys()
                .is_ok()
            );
        }
        for show in [
            "B",
            "Shift+B",
            "Ctrl",
            "Ctrl++B",
            "Ctrl+B+Alt",
            "Ctrl+B+P",
            "Ctrl+AudioVolumeMute",
            "Ctrl+Unknown",
            "Ctrl+Alt+B\n",
            "",
        ] {
            assert!(
                Config {
                    show: show.into(),
                    ..Default::default()
                }
                .keys()
                .is_err(),
                "{show}"
            );
        }
        assert!(
            Config {
                show: "control+option+KeyP".into(),
                ..Default::default()
            }
            .keys()
            .is_err(),
            "aliases must not bypass duplicate check"
        );
        assert!(
            Config {
                show: "B".repeat(100),
                ..Default::default()
            }
            .keys()
            .is_err()
        );
    }
    #[test]
    fn replacement_reuses_swapped_keys_and_drop_releases_all() {
        let backend = Fake::default();
        let owned = backend.owned.clone();
        let mut registry = Registry {
            backend,
            keys: Vec::new(),
        };
        let initial = Config::default().keys().unwrap();
        registry.replace(initial).unwrap();
        registry.replace([initial[1], initial[0]]).unwrap();
        registry.replace([initial[1], initial[0]]).unwrap();
        assert_eq!(owned.borrow().len(), 2);
        registry.replace(keys("Ctrl+Alt+R", "Ctrl+Alt+S")).unwrap();
        assert!(!owned.borrow().contains(&initial[0].id()));
        drop(registry);
        assert!(owned.borrow().is_empty());
    }
    #[test]
    fn failed_second_registration_preserves_previous_shortcuts() {
        let backend = Fake::default();
        let mut registry = Registry {
            backend,
            keys: Vec::new(),
        };
        let initial = Config::default().keys().unwrap();
        registry.replace(initial).unwrap();
        let next = keys("Ctrl+Alt+R", "Ctrl+Alt+S");
        registry.backend.blocked = Some(next[1].id());
        assert!(registry.replace(next).unwrap_err().preserved);
        assert_eq!(registry.keys, initial);
        assert_eq!(
            *registry.backend.owned.borrow(),
            initial.map(|k| k.id()).into_iter().collect()
        );
    }
    #[test]
    fn cleanup_failure_is_reported_and_drop_retries_release() {
        for during_rollback in [false, true] {
            let backend = Fake::default();
            let owned = backend.owned.clone();
            let mut registry = Registry {
                backend,
                keys: Vec::new(),
            };
            let initial = Config::default().keys().unwrap();
            registry.replace(initial).unwrap();
            let next = keys("Ctrl+Alt+R", "Ctrl+Alt+S");
            if during_rollback {
                registry.backend.blocked = Some(next[1].id());
                registry.backend.unregister_failure = Some(next[0].id());
            } else {
                registry.backend.unregister_failure = Some(initial[0].id());
            }
            assert!(!registry.replace(next).unwrap_err().preserved);
            drop(registry);
            assert!(owned.borrow().is_empty());
        }
    }
    #[test]
    fn routing_ignores_unknown_releases_and_repeat_until_key_up() {
        let keys = Config::default().keys().unwrap();
        let mut routing = Routing::default();
        let pressed = |id| GlobalHotKeyEvent {
            id,
            state: HotKeyState::Pressed,
        };
        assert!(routing.event(pressed(keys[0].id())).is_none());
        routing.set(Some(keys));
        assert!(routing.event(pressed(u32::MAX)).is_none());
        assert_eq!(
            routing.event(pressed(keys[0].id())).unwrap().1,
            Action::Recover
        );
        assert_eq!(
            routing.event(pressed(keys[1].id())).unwrap().1,
            Action::Pause
        );
        assert!(routing.event(pressed(keys[1].id())).is_none());
        assert!(
            routing
                .event(GlobalHotKeyEvent {
                    id: keys[1].id(),
                    state: HotKeyState::Released
                })
                .is_none()
        );
        assert_eq!(
            routing.event(pressed(keys[1].id())).unwrap().1,
            Action::Pause
        );
        routing.set(None);
        assert!(routing.event(pressed(keys[0].id())).is_none());
    }
    #[test]
    fn pending_actions_are_discarded_after_rebinding_or_disabling() {
        let mut hotkeys = Hotkeys::load(None);
        hotkeys.config.enabled = true;
        let (sender, receiver) = sync_channel(16);
        hotkeys.receiver = Some(receiver);
        let keys = Config::default().keys().unwrap();
        hotkeys.routing.lock().unwrap().set(Some(keys));
        let generation = hotkeys.routing.lock().unwrap().generation;
        sender.try_send((generation, Action::Pause)).unwrap();
        assert_eq!(hotkeys.pop(), Some(Action::Pause));
        sender.try_send((generation, Action::Recover)).unwrap();
        hotkeys
            .routing
            .lock()
            .unwrap()
            .set(Some([keys[1], keys[0]]));
        assert_eq!(hotkeys.pop(), None);
        let generation = hotkeys.routing.lock().unwrap().generation;
        sender.try_send((generation, Action::Pause)).unwrap();
        hotkeys.disable();
        assert_eq!(hotkeys.pop(), None);
    }
    #[test]
    fn invalid_editor_changes_do_not_replace_a_working_configuration() {
        let mut hotkeys = Hotkeys::load(None);
        hotkeys.request = None;
        hotkeys.config.enabled = true;
        let previous = hotkeys.config.clone();
        hotkeys.draft.pause = hotkeys.draft.show.clone();
        hotkeys.request(true);
        assert!(hotkeys.error.is_some());
        assert!(hotkeys.request.is_none());
        assert_eq!(hotkeys.config, previous);
        hotkeys.draft.pause = "Ctrl+Alt+S".into();
        hotkeys.request(true);
        assert!(hotkeys.error.is_none());
        assert_eq!(hotkeys.request.as_ref().unwrap().pause, "Ctrl+Alt+S");
    }
    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires a real X11 server; CI runs this in Xvfb"]
    fn native_x11_hotkey_registration_conflicts_and_release() {
        let first = global_hotkey::GlobalHotKeyManager::new().unwrap();
        let second = global_hotkey::GlobalHotKeyManager::new().unwrap();
        let key = "Ctrl+Alt+B".parse::<HotKey>().unwrap();
        first.register(key).unwrap();
        assert!(
            second.register(key).is_err(),
            "occupied global hotkey must be rejected"
        );
        first.unregister(key).unwrap();
        second.register(key).unwrap();
        second.unregister(key).unwrap();
    }
}
