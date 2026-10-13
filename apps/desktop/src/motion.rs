//! Desktop-only system preferences. The core receives ordinary Settings.
use std::time::{Duration, Instant};

pub struct MotionPreference {
    pub follow_system: bool,
    pub system: Option<bool>,
}

impl MotionPreference {
    pub fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        Self {
            follow_system: storage
                .and_then(|s| eframe::get_value(s, "follow-system-motion"))
                .unwrap_or(true),
            system: None,
        }
    }

    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "follow-system-motion", &self.follow_system);
    }

    pub fn reduced(&self, manual: bool) -> bool {
        manual || (self.follow_system && self.system == Some(true))
    }

    pub fn status(&self) -> &'static str {
        if !self.follow_system {
            "Deine manuelle Auswahl gilt."
        } else {
            match self.system {
                Some(true) => "System: Weniger Bewegung ist aktiv.",
                Some(false) => "System: Animationen sind erlaubt.",
                None => "Systemeinstellung nicht verfügbar. Deine Auswahl gilt.",
            }
        }
    }
}

// Default is an inert source for headless UI tests. Only the native app starts it.
#[derive(Default)]
pub struct SystemMotion {
    active: bool,
    last_poll: Option<Instant>,
    #[cfg(target_os = "linux")]
    portal: Option<linux::PortalWatcher>,
}

impl SystemMotion {
    pub fn start() -> Self {
        Self {
            active: true,
            ..Default::default()
        }
    }

    /// None means no new sample; Some(None) means the source is unavailable.
    pub fn poll(&mut self, ctx: &eframe::egui::Context, now: Instant) -> Option<Option<bool>> {
        if !self.active
            || self
                .last_poll
                .is_some_and(|last| now.duration_since(last) < Duration::from_secs(1))
        {
            return None;
        }
        self.last_poll = Some(now);
        #[cfg(target_os = "linux")]
        {
            let portal = self
                .portal
                .get_or_insert_with(|| linux::PortalWatcher::start(ctx.clone()));
            Some(portal.value())
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = ctx;
            Some(native_value())
        }
    }
}

#[cfg(target_os = "macos")]
fn native_value() -> Option<bool> {
    // Called on eframe's main thread, without changing any global preference.
    objc2::MainThreadMarker::new()?;
    Some(objc2_app_kit::NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion())
}

#[cfg(target_os = "windows")]
fn native_value() -> Option<bool> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SPI_GETCLIENTAREAANIMATION, SystemParametersInfoW,
    };
    let mut enabled: i32 = 1;
    // SAFETY: SPI_GETCLIENTAREAANIMATION writes exactly one Win32 BOOL.
    let success = unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            (&mut enabled as *mut i32).cast(),
            0,
        )
    };
    (success != 0).then_some(enabled == 0)
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn native_value() -> Option<bool> {
    None
}

#[cfg(target_os = "linux")]
mod linux {
    use std::sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
        mpsc::{self, Sender},
    };
    use std::time::Duration;
    use zbus::{
        blocking::{Connection, Proxy, connection::Builder},
        zvariant::OwnedValue,
    };

    pub(super) struct PortalWatcher {
        value: Arc<AtomicU8>,
        stop: Sender<()>,
    }

    impl PortalWatcher {
        pub(super) fn start(ctx: eframe::egui::Context) -> Self {
            let value = Arc::new(AtomicU8::new(0));
            let current = value.clone();
            let (stop, stopped) = mpsc::channel();
            // Bus connection and queries stay off the GUI thread. The connection
            // is reused; absent services are retried without warning popups.
            let _ = std::thread::Builder::new()
                .name("beni-motion".into())
                .spawn(move || {
                    let mut connection = None;
                    loop {
                        if connection.is_none() {
                            connection = Builder::session()
                                .and_then(|b| b.method_timeout(Duration::from_secs(1)).build())
                                .ok();
                        }
                        let preference = connection.as_ref().and_then(read);
                        let encoded = match preference {
                            None => 0,
                            Some(false) => 1,
                            Some(true) => 2,
                        };
                        if current.swap(encoded, Ordering::Relaxed) != encoded {
                            ctx.request_repaint();
                        }
                        if connection.as_ref().is_some_and(Connection::is_closed) {
                            connection = None;
                        }
                        if !matches!(
                            stopped.recv_timeout(Duration::from_secs(5)),
                            Err(mpsc::RecvTimeoutError::Timeout)
                        ) {
                            break;
                        }
                    }
                });
            Self { value, stop }
        }

        pub(super) fn value(&self) -> Option<bool> {
            match self.value.load(Ordering::Relaxed) {
                1 => Some(false),
                2 => Some(true),
                _ => None,
            }
        }
    }

    impl Drop for PortalWatcher {
        fn drop(&mut self) {
            let _ = self.stop.send(());
        }
    }

    fn read(connection: &Connection) -> Option<bool> {
        let proxy = Proxy::new(
            connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Settings",
        )
        .ok()?;
        let value: OwnedValue = proxy
            .call("ReadOne", &("org.freedesktop.appearance", "reduced-motion"))
            .ok()?;
        let preference = u32::try_from(value).ok()?;
        // The portal specifies unknown numeric values as no preference.
        Some(preference == 1)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::{
            io::{BufRead, BufReader},
            path::PathBuf,
            process::{Child, Command, Stdio},
            sync::atomic::AtomicU32,
        };

        struct PrivateBus {
            child: Child,
            config: PathBuf,
        }
        impl Drop for PrivateBus {
            fn drop(&mut self) {
                let _ = self.child.kill();
                let _ = self.child.wait();
                let _ = std::fs::remove_file(&self.config);
            }
        }

        struct Settings {
            value: Arc<AtomicU32>,
        }
        #[zbus::interface(name = "org.freedesktop.portal.Settings")]
        impl Settings {
            fn read_one(&self, namespace: &str, key: &str) -> zbus::fdo::Result<OwnedValue> {
                assert_eq!(
                    (namespace, key),
                    ("org.freedesktop.appearance", "reduced-motion")
                );
                match self.value.load(Ordering::Relaxed) {
                    u32::MAX => Err(zbus::fdo::Error::UnknownProperty("Missing key".into())),
                    value if value == u32::MAX - 1 => Ok(OwnedValue::from(true)),
                    value => Ok(OwnedValue::from(value)),
                }
            }
        }

        #[test]
        fn private_portal_round_trip_reads_changes_and_rejects_missing_or_wrong_values() {
            let config =
                std::env::temp_dir().join(format!("beni-motion-bus-{}.conf", std::process::id()));
            // No service directories: this private test bus cannot activate host portals.
            std::fs::write(&config, r#"<busconfig><type>session</type><listen>unix:tmpdir=/tmp</listen><auth>EXTERNAL</auth><policy context="default"><allow user="*"/><allow own="*"/><allow send_destination="*"/><allow receive_sender="*"/></policy></busconfig>"#).unwrap();
            let child = Command::new("dbus-daemon")
                .arg(format!("--config-file={}", config.display()))
                .args(["--nofork", "--print-address=1"])
                .stdout(Stdio::piped())
                .spawn()
                .expect("Linux integration test requires dbus-daemon");
            let mut bus = PrivateBus { child, config };
            let mut address = String::new();
            BufReader::new(bus.child.stdout.take().unwrap())
                .read_line(&mut address)
                .unwrap();
            let value = Arc::new(AtomicU32::new(0));
            let service = Builder::address(address.trim())
                .unwrap()
                .name("org.freedesktop.portal.Desktop")
                .unwrap()
                .serve_at(
                    "/org/freedesktop/portal/desktop",
                    Settings {
                        value: value.clone(),
                    },
                )
                .unwrap()
                .build()
                .unwrap();
            let client = Builder::address(address.trim())
                .unwrap()
                .method_timeout(Duration::from_secs(1))
                .build()
                .unwrap();
            for (state, expected) in [
                (0, Some(false)),
                (1, Some(true)),
                (99, Some(false)),
                (u32::MAX, None),
                (u32::MAX - 1, None),
                (1, Some(true)),
            ] {
                value.store(state, Ordering::Relaxed);
                assert_eq!(read(&client), expected, "portal state {state}");
            }
            service.close().unwrap();
            assert_eq!(read(&client), None, "absent portal must fall back");
        }
    }
}
