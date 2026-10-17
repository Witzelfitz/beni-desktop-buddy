//! Native input regions and passive macOS mouse routing.
use crate::hit_map::HitMap;
use eframe::egui;
use std::sync::Arc;
use winit::window::Window;

#[derive(Default)]
pub struct InputGuard {
    #[cfg(not(target_os = "macos"))]
    last: Option<Vec<[i32; 4]>>,
    #[cfg(not(target_os = "macos"))]
    last_scale: Option<(u32, u64)>,
    disabled: bool,
    #[cfg(target_os = "linux")]
    wayland: Option<linux::Wayland>,
    #[cfg(target_os = "macos")]
    monitor: Option<macos::Monitor>,
}
impl InputGuard {
    pub fn retry_if_failed(&mut self) {
        if self.disabled {
            *self = Self::default();
        }
    }
    pub fn apply(
        &mut self,
        window: &Arc<Window>,
        map: &HitMap,
        ctx: &egui::Context,
        enabled: bool,
    ) -> Result<(), &'static str> {
        if self.disabled {
            return Ok(());
        }
        #[cfg(target_os = "macos")]
        {
            if self.monitor.is_none() && enabled {
                self.monitor = macos::Monitor::new(window.clone(), ctx.clone());
                if self.monitor.is_none() {
                    self.disabled = true;
                    return Err("Durchklicken ist nicht verfügbar; Beni bleibt normal bedienbar.");
                }
            }
            if let Some(monitor) = &self.monitor {
                monitor.update(map, ctx.pixels_per_point(), enabled);
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let scale = ctx.pixels_per_point();
            let rectangles = if enabled {
                map.rectangles(scale)
            } else {
                let size = window.inner_size();
                vec![[0, 0, size.width as i32, size.height as i32]]
            };
            let scale_key = (scale.to_bits(), window.scale_factor().to_bits());
            if self.last.as_ref() == Some(&rectangles) && self.last_scale == Some(scale_key) {
                return Ok(());
            }
            #[cfg(target_os = "windows")]
            let result = windows::set(window, &rectangles);
            #[cfg(target_os = "linux")]
            let result = linux::set(window, map, scale, enabled, &rectangles, &mut self.wayland);
            #[cfg(not(any(target_os = "windows", target_os = "linux")))]
            let result: Result<(), ()> = Err(());
            if result.is_err() {
                self.disabled = true;
                #[cfg(target_os = "windows")]
                windows::reset(window);
                let _ = window.set_cursor_hittest(true);
                return Err("Durchklicken ist nicht verfügbar; Beni bleibt normal bedienbar.");
            } else {
                self.last = Some(rectangles);
                self.last_scale = Some(scale_key);
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use objc2::{rc::Retained, runtime::AnyObject};
    use objc2_app_kit::{NSEvent, NSEventMask};
    use std::{
        ptr::NonNull,
        sync::{
            Mutex,
            atomic::{AtomicBool, Ordering},
        },
    };

    struct Snapshot {
        map: HitMap,
        scale: f32,
        enabled: bool,
    }
    pub struct Monitor {
        objects: Vec<Retained<AnyObject>>,
        snapshot: Arc<Mutex<Snapshot>>,
        window: Arc<Window>,
        passthrough: Arc<AtomicBool>,
    }
    impl Monitor {
        pub fn new(window: Arc<Window>, ctx: egui::Context) -> Option<Self> {
            let snapshot = Arc::new(Mutex::new(Snapshot {
                map: HitMap::default(),
                scale: 1.0,
                enabled: false,
            }));
            let passthrough = Arc::new(AtomicBool::new(false));
            let mask =
                NSEventMask::MouseMoved | NSEventMask::LeftMouseDragged | NSEventMask::LeftMouseUp;
            let global = {
                let (window, snapshot, passthrough, ctx) = (
                    window.clone(),
                    snapshot.clone(),
                    passthrough.clone(),
                    ctx.clone(),
                );
                let callback = block2::RcBlock::new(move |_: NonNull<NSEvent>| {
                    update_route(&window, &snapshot, &passthrough, &ctx);
                });
                NSEvent::addGlobalMonitorForEventsMatchingMask_handler(mask, &callback)?
            };
            let local = {
                let (window, snapshot, passthrough) =
                    (window.clone(), snapshot.clone(), passthrough.clone());
                let callback = block2::RcBlock::new(move |event: NonNull<NSEvent>| {
                    update_route(&window, &snapshot, &passthrough, &ctx);
                    event.as_ptr()
                });
                // The local monitor forwards the original event unchanged.
                unsafe { NSEvent::addLocalMonitorForEventsMatchingMask_handler(mask, &callback) }
            };
            let Some(local) = local else {
                // SAFETY: this object is the monitor returned by NSEvent above.
                unsafe {
                    NSEvent::removeMonitor(&global);
                }
                return None;
            };
            Some(Self {
                objects: vec![global, local],
                snapshot,
                window,
                passthrough,
            })
        }
        pub fn update(&self, map: &HitMap, scale: f32, enabled: bool) {
            if let Ok(mut snapshot) = self.snapshot.lock() {
                if snapshot.map != *map {
                    snapshot.map = map.clone();
                }
                snapshot.scale = scale;
                snapshot.enabled = enabled;
            }
            if !enabled {
                self.passthrough.store(false, Ordering::Relaxed);
                let _ = self.window.set_cursor_hittest(true);
            } else {
                route(&self.window, &self.snapshot, &self.passthrough);
            }
        }
    }
    impl Drop for Monitor {
        fn drop(&mut self) {
            for monitor in &self.objects {
                // SAFETY: only objects returned by NSEvent's monitor methods are retained here.
                unsafe {
                    NSEvent::removeMonitor(monitor);
                }
            }
            let _ = self.window.set_cursor_hittest(true);
        }
    }
    fn update_route(
        window: &Window,
        snapshot: &Mutex<Snapshot>,
        passthrough: &AtomicBool,
        ctx: &egui::Context,
    ) {
        let before = passthrough.load(Ordering::Relaxed);
        route(window, snapshot, passthrough);
        if before != passthrough.load(Ordering::Relaxed) {
            ctx.request_repaint();
        }
    }
    fn route(window: &Window, snapshot: &Mutex<Snapshot>, passthrough: &AtomicBool) {
        // Preserve the native target for the entire press/drag. Monitors run on the main thread.
        if NSEvent::pressedMouseButtons() & 1 != 0 {
            return;
        }
        let Ok(snapshot) = snapshot.lock() else {
            return;
        };
        if !snapshot.enabled || !window.is_visible().unwrap_or(true) {
            return;
        }
        let Some(point) = cursor(window, snapshot.scale) else {
            return;
        };
        let ignore = !snapshot.map.contains(point);
        if passthrough.swap(ignore, Ordering::Relaxed) != ignore {
            let _ = window.set_cursor_hittest(!ignore);
        }
    }
    pub fn cursor(window: &Window, pixels_per_point: f32) -> Option<egui::Pos2> {
        use objc2::MainThreadMarker;
        use objc2_app_kit::NSScreen;
        let screens = NSScreen::screens(MainThreadMarker::new()?);
        if screens.is_empty() {
            return None;
        }
        let primary = screens.objectAtIndex(0).frame();
        let cursor = NSEvent::mouseLocation();
        let origin = window.inner_position().ok()?;
        let scale = window.scale_factor();
        Some(egui::pos2(
            ((cursor.x * scale - f64::from(origin.x)) / f64::from(pixels_per_point)) as f32,
            (((primary.origin.y + primary.size.height - cursor.y) * scale - f64::from(origin.y))
                / f64::from(pixels_per_point)) as f32,
        ))
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use windows_sys::Win32::Graphics::Gdi::{DeleteObject, ExtCreateRegion, RGNDATA, SetWindowRgn};
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    pub fn reset(window: &Window) {
        if let Ok(handle) = window.window_handle()
            && let RawWindowHandle::Win32(handle) = handle.as_raw()
        {
            // SAFETY: borrowed HWND is live; a null region restores the whole window.
            unsafe {
                SetWindowRgn(handle.hwnd.get() as _, std::ptr::null_mut(), 1);
            }
        }
    }
    pub fn set(window: &Window, rectangles: &[[i32; 4]]) -> Result<(), ()> {
        let RawWindowHandle::Win32(handle) = window.window_handle().map_err(|_| ())?.as_raw()
        else {
            return Err(());
        };
        let size = (rectangles.len() * 16) as i32;
        let bounds = rectangles.first().copied().unwrap_or([0; 4]);
        let mut data = vec![
            32,
            1,
            rectangles.len() as i32,
            size,
            bounds[0],
            bounds[1],
            bounds[2],
            bounds[3],
        ];
        for r in rectangles {
            data[4] = data[4].min(r[0]);
            data[5] = data[5].min(r[1]);
            data[6] = data[6].max(r[2]);
            data[7] = data[7].max(r[3]);
            data.extend_from_slice(r);
        }
        // RGNDATA contains a DWORD header and RECTs (all 32-bit fields, aligned by Vec<i32>).
        unsafe {
            let region = ExtCreateRegion(
                std::ptr::null(),
                (data.len() * 4) as u32,
                data.as_ptr().cast::<RGNDATA>(),
            );
            if region.is_null() {
                return Err(());
            }
            if SetWindowRgn(handle.hwnd.get() as _, region, 1) == 0 {
                DeleteObject(region);
                return Err(());
            }
            // Windows owns the region after a successful SetWindowRgn call.
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod linux;
