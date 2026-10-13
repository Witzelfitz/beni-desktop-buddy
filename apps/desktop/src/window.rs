use crate::placement::{Bounds, recovered_position};
use winit::window::Window;
pub struct PlacementGuard {
    screens: Option<Vec<Bounds>>,
    window_size: Option<[f64; 2]>,
    checked: std::time::Instant,
}
impl Default for PlacementGuard {
    fn default() -> Self {
        Self {
            screens: None,
            window_size: None,
            checked: std::time::Instant::now(),
        }
    }
}
impl PlacementGuard {
    pub fn check(&mut self, window: &Window, reset: bool) {
        let now = std::time::Instant::now();
        if !reset
            && self.screens.is_some()
            && now.duration_since(self.checked) < std::time::Duration::from_secs(1)
        {
            return;
        }
        self.checked = now;
        let Some((bounds, screens)) = geometry(window) else {
            return;
        };
        // An initial zero-sized window must not consume the startup recovery check.
        if !bounds.valid() || !screens.iter().any(|screen| screen.valid()) {
            return;
        }
        let changed = self.screens.as_ref() != Some(&screens);
        let size = [bounds.width, bounds.height];
        let resized = self.window_size != Some(size);
        let lost = !screens.iter().any(|s| {
            bounds.x + bounds.width > s.x
                && bounds.x < s.x + s.width
                && bounds.y + bounds.height > s.y
                && bounds.y < s.y + s.height
        });
        if (reset || changed || resized || lost)
            && let Some(position) = recovered_position(bounds, &screens, reset)
        {
            place(window, position);
        }
        self.screens = Some(screens);
        self.window_size = Some(size);
    }
}
#[cfg(target_os = "macos")]
fn geometry(window: &Window) -> Option<(Bounds, Vec<Bounds>)> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSScreen;
    let all = NSScreen::screens(MainThreadMarker::new()?);
    if all.is_empty() {
        return None;
    }
    let primary = all.objectAtIndex(0).frame();
    let top = primary.origin.y + primary.size.height;
    let screens = all
        .iter()
        .map(|screen| {
            let f = screen.visibleFrame();
            Bounds {
                x: f.origin.x,
                y: top - f.origin.y - f.size.height,
                width: f.size.width,
                height: f.size.height,
            }
        })
        .collect();
    let scale = window.scale_factor();
    let p = window.outer_position().ok()?.to_logical::<f64>(scale);
    let s = window.outer_size().to_logical::<f64>(scale);
    Some((
        Bounds {
            x: p.x,
            y: p.y,
            width: s.width,
            height: s.height,
        },
        screens,
    ))
}
#[cfg(not(target_os = "macos"))]
fn geometry(window: &Window) -> Option<(Bounds, Vec<Bounds>)> {
    let primary = window.primary_monitor();
    let mut monitors: Vec<_> = window.available_monitors().collect();
    monitors.sort_by_key(|m| Some(m) != primary.as_ref());
    let screens = monitors
        .iter()
        .map(|m| {
            let p = m.position();
            let s = m.size();
            Bounds {
                x: f64::from(p.x),
                y: f64::from(p.y),
                width: f64::from(s.width),
                height: f64::from(s.height),
            }
        })
        .collect();
    let p = window.outer_position().ok()?;
    let s = window.outer_size();
    Some((
        Bounds {
            x: f64::from(p.x),
            y: f64::from(p.y),
            width: f64::from(s.width),
            height: f64::from(s.height),
        },
        screens,
    ))
}
#[cfg(target_os = "macos")]
fn place(window: &Window, p: [f64; 2]) {
    window.set_outer_position(winit::dpi::LogicalPosition::new(p[0], p[1]));
}
#[cfg(not(target_os = "macos"))]
fn place(window: &Window, p: [f64; 2]) {
    window.set_outer_position(winit::dpi::PhysicalPosition::new(
        p[0].round() as i32,
        p[1].round() as i32,
    ));
}
pub fn restore(window: &Window, guard: &mut PlacementGuard, reset: bool) {
    window.set_visible(true);
    window.set_minimized(false);
    guard.check(window, reset);
    window.focus_window();
}
pub fn tray_unavailable(window: &Window) {
    window.set_visible(true);
    window.set_decorations(true);
    #[cfg(target_os = "macos")]
    if let Some(mtm) = objc2::MainThreadMarker::new() {
        use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
        NSApplication::sharedApplication(mtm)
            .setActivationPolicy(NSApplicationActivationPolicy::Regular);
    }
}
