//! Platform updater lives in the desktop; the animation core stays network-free.
use eframe::egui;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Status {
    #[default]
    NotStarted,
    #[cfg(target_os = "macos")]
    Ready,
    #[cfg(target_os = "macos")]
    PendingFeed,
    #[cfg(target_os = "macos")]
    Unavailable,
    #[cfg(not(target_os = "macos"))]
    Unsupported,
}

#[derive(Default)]
pub struct Updater {
    status: Status,
    install: InstallGate,
    pending_install: bool,
    #[cfg(target_os = "macos")]
    native: Option<macos::Native>,
}

#[derive(Default)]
struct InstallGate {
    deferred: bool,
    confirmed: bool,
}
impl InstallGate {
    fn ready(&self, pending: bool, busy: bool) -> bool {
        pending && (self.confirmed || (!self.deferred && !busy))
    }
}

impl Updater {
    /// Call once a real native window exists. Headless tests never load Sparkle.
    pub fn initialize(&mut self) {
        if self.status != Status::NotStarted {
            return;
        }
        #[cfg(target_os = "macos")]
        {
            match macos::Native::new() {
                Some((native, status)) => {
                    self.native = Some(native);
                    self.status = status;
                }
                None => self.status = Status::Unavailable,
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.status = Status::Unsupported;
        }
    }

    pub fn can_check(&self) -> bool {
        #[cfg(target_os = "macos")]
        return self.status == Status::Ready
            && self.native.as_ref().is_some_and(macos::Native::can_check);
        #[cfg(not(target_os = "macos"))]
        false
    }

    pub fn check(&mut self) -> Result<(), &'static str> {
        if !self.can_check() {
            return Err(self.status_text());
        }
        #[cfg(target_os = "macos")]
        if let Some(native) = &self.native {
            native.check();
        }
        Ok(())
    }
    pub fn menu_enabled(&self) -> bool {
        !self.is_ready() || self.can_check()
    }

    fn status_text(&self) -> &'static str {
        match self.status {
            #[cfg(target_os = "macos")]
            Status::PendingFeed => {
                "Updates über Beni sind vorbereitet. Bis zur Freigabe findest du neue Versionen auf mini.witzelfitz.io/download."
            }
            #[cfg(not(target_os = "macos"))]
            Status::Unsupported => {
                "Updates über Beni sind auf dieser Plattform noch in Vorbereitung. Neue Versionen findest du auf mini.witzelfitz.io/download."
            }
            Status::NotStarted => "Die Updatefunktion ist noch nicht gestartet.",
            #[cfg(target_os = "macos")]
            Status::Unavailable => {
                "Die Updatefunktion benötigt das vollständige macOS-App-Bundle. Neue Versionen findest du auf mini.witzelfitz.io/download."
            }
            #[cfg(target_os = "macos")]
            Status::Ready => {
                "Die Updateprüfung läuft bereits. Bitte schliesse zuerst das Updatefenster."
            }
        }
    }

    pub fn poll(&mut self) {
        #[cfg(target_os = "macos")]
        {
            self.pending_install = self
                .native
                .as_ref()
                .is_some_and(macos::Native::install_pending);
            if !self.pending_install {
                self.install = InstallGate::default();
            }
        }
    }
    fn install_pending(&self) -> bool {
        self.pending_install
    }

    pub fn needs_confirmation(&self, busy: bool) -> bool {
        self.install_pending() && !self.install.ready(true, busy) && !self.install.deferred
    }

    pub fn ready_to_install(&self, busy: bool) -> bool {
        self.install.ready(self.install_pending(), busy)
    }
    pub fn defer_install(&mut self) {
        self.install.deferred = true;
        self.install.confirmed = false;
    }

    /// Caller saves/flushes storage and stops IPC before this handoff.
    pub fn finish_install(&mut self) {
        #[cfg(target_os = "macos")]
        if let Some(native) = &self.native {
            native.finish_install();
        }
        self.install = InstallGate::default();
        self.pending_install = false;
    }

    fn is_ready(&self) -> bool {
        #[cfg(target_os = "macos")]
        return self.status == Status::Ready;
        #[cfg(not(target_os = "macos"))]
        false
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.heading("Updates");
        if ui
            .add_enabled(
                self.menu_enabled(),
                egui::Button::new("Nach Updates suchen …"),
            )
            .clicked()
            && let Err(message) = self.check()
        {
            ui.label(message);
        }
        if !self.is_ready() {
            ui.small(self.status_text());
            ui.hyperlink_to(
                "Downloadseite öffnen",
                "https://mini.witzelfitz.io/download",
            );
        }
        #[cfg(target_os = "macos")]
        if let Some(native) = &self.native {
            let mut automatic = native.automatic();
            if ui
                .add_enabled(
                    self.status == Status::Ready,
                    egui::Checkbox::new(&mut automatic, "Automatisch nach Updates suchen"),
                )
                .changed()
            {
                // Sparkle persists this preference itself. Do not overwrite it
                // on startup or maintain a second copy in eframe storage.
                native.set_automatic(automatic);
            }
        }
        if self.install_pending() {
            ui.label(
                "Update bereit. Ein Neustart beendet laufende Timer und geplante Erinnerungen.",
            );
            ui.horizontal(|ui| {
                if ui.button("Später").clicked() {
                    self.defer_install();
                }
                if ui.button("Update installieren und neu starten").clicked() {
                    self.install.confirmed = true;
                }
            });
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::Status;
    use std::{ffi::c_void, marker::PhantomData, ptr::NonNull, rc::Rc};

    unsafe extern "C" {
        fn beni_updater_create(status: *mut i32) -> *mut c_void;
        fn beni_updater_can_check(handle: *mut c_void) -> i32;
        fn beni_updater_check(handle: *mut c_void);
        fn beni_updater_automatic(handle: *mut c_void) -> i32;
        fn beni_updater_set_automatic(handle: *mut c_void, enabled: i32);
        fn beni_updater_install_pending(handle: *mut c_void) -> i32;
        fn beni_updater_finish_install(handle: *mut c_void);
        fn beni_updater_destroy(handle: *mut c_void);
    }

    pub(super) struct Native {
        handle: NonNull<c_void>,
        // Cocoa instances never move to another thread.
        _main_thread: PhantomData<Rc<()>>,
    }
    impl Native {
        pub(super) fn new() -> Option<(Self, Status)> {
            objc2::MainThreadMarker::new()?;
            let mut status = 0;
            // SAFETY: main thread; valid status pointer. Bridge returns a retained handle.
            let handle = NonNull::new(unsafe { beni_updater_create(&mut status) })?;
            Some((
                Self {
                    handle,
                    _main_thread: PhantomData,
                },
                if status == 1 {
                    Status::Ready
                } else {
                    Status::PendingFeed
                },
            ))
        }
        pub(super) fn can_check(&self) -> bool {
            // SAFETY: live main-thread handle owned by this wrapper.
            unsafe { beni_updater_can_check(self.handle.as_ptr()) != 0 }
        }
        pub(super) fn check(&self) {
            // SAFETY: live main-thread handle; bridge also guards session state.
            unsafe { beni_updater_check(self.handle.as_ptr()) }
        }
        pub(super) fn automatic(&self) -> bool {
            // SAFETY: live main-thread handle.
            unsafe { beni_updater_automatic(self.handle.as_ptr()) != 0 }
        }
        pub(super) fn set_automatic(&self, enabled: bool) {
            // SAFETY: live main-thread handle, integer boolean.
            unsafe { beni_updater_set_automatic(self.handle.as_ptr(), i32::from(enabled)) }
        }
        pub(super) fn install_pending(&self) -> bool {
            // SAFETY: live main-thread handle.
            unsafe { beni_updater_install_pending(self.handle.as_ptr()) != 0 }
        }
        pub(super) fn finish_install(&self) {
            // SAFETY: live main-thread handle; callback runs after this returns.
            unsafe { beni_updater_finish_install(self.handle.as_ptr()) }
        }
    }
    impl Drop for Native {
        fn drop(&mut self) {
            // SAFETY: release exactly once, on the owning main thread.
            unsafe { beni_updater_destroy(self.handle.as_ptr()) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_requires_a_request_and_explicit_confirmation_when_busy_or_deferred() {
        let mut gate = InstallGate::default();
        assert!(!gate.ready(false, false));
        assert!(gate.ready(true, false));
        assert!(!gate.ready(true, true));
        gate.deferred = true;
        assert!(
            !gate.ready(true, false),
            "finishing tasks must not undo Später"
        );
        gate.confirmed = true;
        assert!(gate.ready(true, true));
        assert!(!gate.ready(false, false));
    }

    #[test]
    fn unavailable_or_unstarted_updater_never_checks_or_installs() {
        for status in [
            Status::NotStarted,
            #[cfg(target_os = "macos")]
            Status::Unavailable,
            #[cfg(target_os = "macos")]
            Status::PendingFeed,
            #[cfg(not(target_os = "macos"))]
            Status::Unsupported,
        ] {
            let mut updater = Updater {
                status,
                ..Default::default()
            };
            assert!(!updater.can_check());
            assert!(updater.check().is_err());
            assert!(!updater.ready_to_install(false));
            assert!(!updater.needs_confirmation(true));
        }
    }

    #[test]
    fn real_update_buttons_defer_and_confirm_the_pending_restart() {
        let ctx = egui::Context::default();
        let mut updater = Updater {
            pending_install: true,
            ..Default::default()
        };
        assert!(updater.needs_confirmation(true));

        fn draw(
            ctx: &egui::Context,
            updater: &mut Updater,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| updater.ui(ui),
            )
        }
        fn click(ctx: &egui::Context, updater: &mut Updater, label: &str) {
            let output = draw(ctx, updater, vec![]);
            let position = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::epaint::Shape::Text(text) if text.galley.job.text == label => {
                        Some(text.pos + text.galley.size() / 2.0)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("Update button missing: {label}"));
            output.drop_without_applying_deltas();
            for pressed in [true, false] {
                draw(
                    ctx,
                    updater,
                    vec![
                        egui::Event::PointerMoved(position),
                        egui::Event::PointerButton {
                            pos: position,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::default(),
                        },
                    ],
                )
                .drop_without_applying_deltas();
            }
        }
        click(&ctx, &mut updater, "Später");
        assert!(
            !updater.ready_to_install(false),
            "no automatic restart after Später, even when tasks finish"
        );
        assert!(
            !updater.needs_confirmation(true),
            "no repeated focus stealing"
        );
        click(&ctx, &mut updater, "Update installieren und neu starten");
        assert!(updater.ready_to_install(true));
        updater.defer_install();
        assert!(
            !updater.ready_to_install(false),
            "a later cancellation revokes a previous confirmation"
        );
        updater.pending_install = false;
        assert!(!updater.ready_to_install(false));
    }
}
