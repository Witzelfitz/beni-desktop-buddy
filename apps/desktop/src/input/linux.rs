use super::*;
use std::{
    ffi::{CStr, c_char, c_int, c_ulong, c_void},
    ptr,
};
use winit::raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};

pub(super) fn set(
    window: &Arc<Window>,
    map: &HitMap,
    scale: f32,
    enabled: bool,
    rectangles: &[[i32; 4]],
    wayland: &mut Option<Wayland>,
) -> Result<(), ()> {
    let display = window.display_handle().map_err(|_| ())?.as_raw();
    let handle = window.window_handle().map_err(|_| ())?.as_raw();
    match (display, handle) {
        (RawDisplayHandle::Xlib(display), RawWindowHandle::Xlib(handle)) => {
            let display = display.display.ok_or(())?;
            let mut event = 0;
            let mut error = 0;
            let mut native = Vec::with_capacity(rectangles.len());
            for r in rectangles {
                native.push(XRectangle {
                    x: i16::try_from(r[0]).map_err(|_| ())?,
                    y: i16::try_from(r[1]).map_err(|_| ())?,
                    width: u16::try_from(r[2] - r[0]).map_err(|_| ())?,
                    height: u16::try_from(r[3] - r[1]).map_err(|_| ())?,
                });
            }
            // SAFETY: handles are borrowed from the live winit window; the XRectangle buffer
            // remains alive through the synchronous call. ShapeInput changes only input geometry.
            unsafe {
                if XShapeQueryExtension(display.as_ptr(), &mut event, &mut error) == 0 {
                    return Err(());
                }
                XShapeCombineRectangles(
                    display.as_ptr(),
                    handle.window,
                    2,
                    0,
                    0,
                    native.as_mut_ptr(),
                    native.len() as c_int,
                    0,
                    0,
                );
                XFlush(display.as_ptr());
            }
            Ok(())
        }
        (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(handle)) => {
            if wayland.is_none() {
                // SAFETY: raw handles are libwayland-client handles borrowed from winit.
                *wayland = unsafe {
                    Wayland::new(
                        window.clone(),
                        display.display.as_ptr(),
                        handle.surface.as_ptr(),
                    )
                };
            }
            let compositor = wayland.as_ref().ok_or(())?;
            let rectangles = if enabled {
                map.rectangles(scale / window.scale_factor() as f32)
            } else {
                let size = window.inner_size().to_logical::<f64>(window.scale_factor());
                vec![[0, 0, size.width.ceil() as i32, size.height.ceil() as i32]]
            };
            compositor.set(&rectangles)
        }
        _ => Err(()),
    }
}

#[repr(C)]
struct XRectangle {
    x: i16,
    y: i16,
    width: u16,
    height: u16,
}
#[link(name = "Xext")]
unsafe extern "C" {
    fn XShapeQueryExtension(display: *mut c_void, event: *mut c_int, error: *mut c_int) -> c_int;
    fn XShapeCombineRectangles(
        display: *mut c_void,
        window: c_ulong,
        kind: c_int,
        x: c_int,
        y: c_int,
        rectangles: *mut XRectangle,
        count: c_int,
        operation: c_int,
        ordering: c_int,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires an X11 or Wayland test server"]
    fn native_linux_input_regions() {
        use winit::platform::{wayland::EventLoopBuilderExtWayland, x11::EventLoopBuilderExtX11};
        use winit::{
            application::ApplicationHandler,
            event::WindowEvent,
            event_loop::{ActiveEventLoop, EventLoop},
            window::WindowId,
        };
        struct Probe(bool);
        impl ApplicationHandler for Probe {
            fn resumed(&mut self, event_loop: &ActiveEventLoop) {
                let window = Arc::new(
                    event_loop
                        .create_window(
                            Window::default_attributes()
                                .with_inner_size(winit::dpi::LogicalSize::new(200, 100))
                                .with_decorations(false)
                                .with_visible(false),
                        )
                        .unwrap(),
                );
                let ctx = egui::Context::default();
                ctx.set_pixels_per_point(1.0);
                let mut map = HitMap::default();
                map.reset(egui::vec2(200.0, 100.0));
                map.add_rect(
                    egui::Rect::from_min_max(egui::pos2(20.0, 20.0), egui::pos2(80.0, 80.0)),
                    0.0,
                );
                let mut guard = InputGuard::default();
                guard
                    .apply(&window, &map, &ctx, true)
                    .expect("native input region");
                probe(&window, true);
                guard
                    .apply(&window, &map, &ctx, false)
                    .expect("restore full input region");
                probe(&window, false);
                guard
                    .apply(&window, &map, &ctx, true)
                    .expect("reactivate selective input region");
                probe(&window, true);
                self.0 = true;
                event_loop.exit();
            }
            fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
        }
        let mut builder = EventLoop::builder();
        EventLoopBuilderExtX11::with_any_thread(&mut builder, true);
        if std::env::var("BENI_INPUT_BACKEND").as_deref() == Ok("wayland") {
            builder.with_wayland();
        } else {
            builder.with_x11();
        }
        let event_loop = builder.build().unwrap();
        let mut app = Probe(false);
        event_loop.run_app(&mut app).unwrap();
        assert!(app.0);
    }
    fn probe(window: &Window, selective: bool) {
        match (
            window.display_handle().unwrap().as_raw(),
            window.window_handle().unwrap().as_raw(),
        ) {
            (RawDisplayHandle::Xlib(display), RawWindowHandle::Xlib(handle)) => {
                // SAFETY: XShapeGetRectangles allocates a buffer owned by the caller; count
                // is checked before reading and the buffer is freed with XFree afterwards.
                unsafe {
                    let mut count = 0;
                    let mut order = 0;
                    let pointer = XShapeGetRectangles(
                        display.display.unwrap().as_ptr(),
                        handle.window,
                        2,
                        &mut count,
                        &mut order,
                    );
                    assert!(!pointer.is_null() && (1..10000).contains(&count));
                    let rectangles = std::slice::from_raw_parts(pointer, count as usize);
                    let contains = |x: i32, y: i32| {
                        rectangles.iter().any(|r| {
                            x >= i32::from(r.x)
                                && y >= i32::from(r.y)
                                && x < i32::from(r.x) + i32::from(r.width)
                                && y < i32::from(r.y) + i32::from(r.height)
                        })
                    };
                    assert!(contains(40, 40));
                    assert_eq!(contains(0, 0), !selective);
                    XFree(pointer.cast());
                }
            }
            (RawDisplayHandle::Wayland(display), _) => unsafe {
                // Roundtrip on an independent queue checks that the compositor accepted
                // the requests without dispatching winit's pending events on its queue.
                let queue = wl_display_create_queue(display.display.as_ptr());
                assert!(!queue.is_null());
                let result = wl_display_roundtrip_queue(display.display.as_ptr(), queue);
                let error = wl_display_get_error(display.display.as_ptr());
                wl_event_queue_destroy(queue);
                assert!(result >= 0);
                assert_eq!(error, 0, "invalid Wayland input-region request");
            },
            _ => panic!("unsupported native test display"),
        }
    }
    #[link(name = "Xext")]
    unsafe extern "C" {
        fn XShapeGetRectangles(
            display: *mut c_void,
            window: c_ulong,
            kind: c_int,
            count: *mut c_int,
            ordering: *mut c_int,
        ) -> *mut XRectangle;
    }
    #[link(name = "X11")]
    unsafe extern "C" {
        fn XFree(pointer: *mut c_void) -> c_int;
    }
    #[link(name = "wayland-client")]
    unsafe extern "C" {
        fn wl_display_get_error(display: *mut c_void) -> c_int;
    }
}
#[link(name = "X11")]
unsafe extern "C" {
    fn XFlush(display: *mut c_void) -> c_int;
}

// Own only the compositor proxy; window/display/surface remain owned by winit. Holding
// the Window Arc keeps the native surface alive until after the compositor proxy is dropped.
pub(super) struct Wayland {
    _window: Arc<Window>,
    display: *mut c_void,
    surface: *mut c_void,
    compositor: *mut c_void,
}
impl Wayland {
    unsafe fn new(window: Arc<Window>, display: *mut c_void, surface: *mut c_void) -> Option<Self> {
        // A dedicated queue avoids dispatching winit's events during discovery. Registry
        // callbacks reference the local `compositor` slot only until this queue is destroyed.
        unsafe {
            let queue = wl_display_create_queue(display);
            if queue.is_null() {
                return None;
            }
            let registry = wl_proxy_marshal_flags(
                display,
                1,
                &raw const wl_registry_interface,
                1,
                0,
                ptr::null_mut::<c_void>(),
            );
            if registry.is_null() {
                wl_event_queue_destroy(queue);
                return None;
            }
            wl_proxy_set_queue(registry, queue);
            let mut compositor = ptr::null_mut::<c_void>();
            let listener = wl_proxy_add_listener(
                registry,
                (&raw const REGISTRY_LISTENER).cast(),
                (&raw mut compositor).cast(),
            );
            let result = if listener == 0 {
                wl_display_roundtrip_queue(display, queue)
            } else {
                -1
            };
            wl_proxy_destroy(registry);
            if !compositor.is_null() {
                wl_proxy_set_queue(compositor, ptr::null_mut());
            }
            wl_event_queue_destroy(queue);
            if result < 0 || compositor.is_null() {
                if !compositor.is_null() {
                    wl_proxy_destroy(compositor);
                }
                return None;
            }
            Some(Self {
                _window: window,
                display,
                surface,
                compositor,
            })
        }
    }
    fn set(&self, rectangles: &[[i32; 4]]) -> Result<(), ()> {
        // SAFETY: owned compositor proxy and borrowed surface share the same live display.
        // Region is copied by set_input_region and can be destroyed immediately afterwards.
        unsafe {
            let region = wl_proxy_marshal_flags(
                self.compositor,
                1,
                &raw const wl_region_interface,
                1,
                0,
                ptr::null_mut::<c_void>(),
            );
            if region.is_null() {
                return Err(());
            }
            for r in rectangles {
                wl_proxy_marshal_flags(
                    region,
                    1,
                    ptr::null(),
                    1,
                    0,
                    r[0],
                    r[1],
                    r[2] - r[0],
                    r[3] - r[1],
                );
            }
            let version = wl_proxy_get_version(self.surface);
            wl_proxy_marshal_flags(self.surface, 5, ptr::null(), version, 0, region);
            wl_proxy_marshal_flags(region, 0, ptr::null(), 1, 1);
            wl_proxy_marshal_flags(self.surface, 6, ptr::null(), version, 0);
            wl_display_flush(self.display);
        }
        Ok(())
    }
}
impl Drop for Wayland {
    fn drop(&mut self) {
        // SAFETY: owned proxy is destroyed before the retained winit window is released.
        unsafe {
            wl_proxy_destroy(self.compositor);
        }
    }
}

#[repr(C)]
struct RegistryListener {
    global: unsafe extern "C" fn(*mut c_void, *mut c_void, u32, *const c_char, u32),
    remove: unsafe extern "C" fn(*mut c_void, *mut c_void, u32),
}
static REGISTRY_LISTENER: RegistryListener = RegistryListener { global, remove };
unsafe extern "C" fn global(
    data: *mut c_void,
    registry: *mut c_void,
    name: u32,
    interface: *const c_char,
    version: u32,
) {
    // SAFETY: libwayland supplies a NUL-terminated interface name and the registered data
    // slot stays valid until the dedicated discovery queue and registry are destroyed.
    unsafe {
        if version >= 1 && CStr::from_ptr(interface).to_bytes() == b"wl_compositor" {
            let slot = &mut *data.cast::<*mut c_void>();
            if slot.is_null() {
                *slot = wl_proxy_marshal_flags(
                    registry,
                    0,
                    &raw const wl_compositor_interface,
                    1,
                    0,
                    name,
                    c"wl_compositor".as_ptr(),
                    1_u32,
                    ptr::null_mut::<c_void>(),
                );
            }
        }
    }
}
unsafe extern "C" fn remove(_: *mut c_void, _: *mut c_void, _: u32) {}

#[link(name = "wayland-client")]
unsafe extern "C" {
    static wl_registry_interface: c_void;
    static wl_compositor_interface: c_void;
    static wl_region_interface: c_void;
    fn wl_proxy_marshal_flags(
        proxy: *mut c_void,
        opcode: u32,
        interface: *const c_void,
        version: u32,
        flags: u32,
        ...
    ) -> *mut c_void;
    fn wl_proxy_add_listener(
        proxy: *mut c_void,
        listener: *const c_void,
        data: *mut c_void,
    ) -> c_int;
    fn wl_proxy_set_queue(proxy: *mut c_void, queue: *mut c_void);
    fn wl_proxy_get_version(proxy: *mut c_void) -> u32;
    fn wl_proxy_destroy(proxy: *mut c_void);
    fn wl_display_create_queue(display: *mut c_void) -> *mut c_void;
    fn wl_event_queue_destroy(queue: *mut c_void);
    fn wl_display_roundtrip_queue(display: *mut c_void, queue: *mut c_void) -> c_int;
    fn wl_display_flush(display: *mut c_void) -> c_int;
}
