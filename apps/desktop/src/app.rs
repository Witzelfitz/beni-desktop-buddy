use crate::{
    tray::{Action, Tray},
    window::PlacementGuard,
};
use buddy_core::{ATLAS_PNG, Engine, Player, Settings, notifications::reading_time_ms};
use eframe::egui::{self, Color32, FontId, Rect, Sense, TextureHandle, Vec2};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::time::{Duration, Instant};

struct TransientError {
    message: String,
    until: Instant,
}

pub struct BuddyApp {
    engine: Engine,
    player: Player,
    texture: TextureHandle,
    settings: Settings,
    last_tick: Instant,
    message: String,
    message_until: Instant,
    message_source: Option<String>,
    active_remote: bool,
    ipc_enabled: bool,
    server: Option<buddy_ipc::Server>,
    hidden: bool,
    menu_open: bool,
    speech_draft: String,
    error: Option<TransientError>,
    last_render_error: Option<String>,
    tray: Option<Tray>,
    tray_attempted: bool,
    tray_sender: SyncSender<Action>,
    tray_receiver: Receiver<Action>,
    placement: PlacementGuard,
}
impl BuddyApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        engine: Engine,
        settings: Settings,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let icon = eframe::icon_data::from_png_bytes(ATLAS_PNG)?;
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [icon.width as usize, icon.height as usize],
            &icon.rgba,
        );
        let texture = cc
            .egui_ctx
            .load_texture("beni-atlas", image, egui::TextureOptions::LINEAR);
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let settings = settings.sanitize();
        cc.egui_ctx
            .send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                if settings.always_on_top {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                },
            ));
        let mut player = Player::default();
        player.play(&engine, "hallo", true)?;
        let (tray_sender, tray_receiver) = sync_channel(32);
        Ok(Self {
            engine,
            player,
            texture,
            settings,
            last_tick: Instant::now(),
            message: "Hoi! Ich bin Beni. Zieh mich an deinen Lieblingsplatz.".into(),
            message_until: Instant::now() + Duration::from_secs(10),
            message_source: None,
            active_remote: false,
            ipc_enabled: cc
                .storage
                .and_then(|s| eframe::get_value(s, "status-receiving"))
                .unwrap_or(false),
            server: None,
            hidden: false,
            menu_open: false,
            speech_draft: "Schön bist du da!".into(),
            error: None,
            last_render_error: None,
            tray: None,
            tray_attempted: false,
            tray_sender,
            tray_receiver,
            placement: PlacementGuard::default(),
        })
    }
    fn play(&mut self, state: &str, message: &str) {
        if let Err(e) = self.player.play(&self.engine, state, true) {
            self.report_error(e.to_string(), Instant::now());
            return;
        }
        self.error = None;
        self.message = message.chars().take(400).collect();
        self.message_until = Instant::now() + Duration::from_millis(reading_time_ms(&self.message));
        self.message_source = None;
        self.active_remote = false;
    }
    fn report_error(&mut self, message: String, now: Instant) {
        self.error = Some(TransientError {
            message,
            until: now + Duration::from_secs(6),
        });
    }
    fn expire_error(&mut self, now: Instant) {
        if self.error.as_ref().is_some_and(|error| now >= error.until) {
            self.error = None;
        }
    }
    fn dismiss_speech(&mut self, now: Instant) {
        self.error = None;
        self.message_until = now;
        self.active_remote = false;
    }
    fn deliver_status(&mut self, visible: bool, now: Instant) {
        if !visible
            || self.menu_open
            || self.error.is_some()
            || (self.active_remote && now < self.message_until)
        {
            return;
        }
        if let Some(delivery) = self.server.as_ref().and_then(|s| s.pop()) {
            let message = delivery.message;
            self.play(message.state.as_deref().unwrap_or("reden"), &message.text);
            self.message_source = Some(message.source);
            self.message_until = now
                + Duration::from_millis(reading_time_ms(&message.text).min(delivery.remaining_ms));
            self.active_remote = true;
        }
    }
    fn speech(&mut self, ui: &mut egui::Ui, now: Instant) {
        if now >= self.message_until && self.error.is_none() {
            ui.allocate_space(Vec2::new(ui.available_width(), 78.0));
            return;
        }
        let top = ui.cursor().min.y;
        let text = self
            .error
            .as_ref()
            .map(|e| &e.message)
            .unwrap_or(&self.message)
            .clone();
        let mut dismiss = false;
        let response = egui::Frame::new()
            .fill(Color32::from_rgb(246, 249, 237))
            .corner_radius(16)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.set_width((ui.available_width() - 24.0).max(100.0));
                ui.visuals_mut().override_text_color = Some(Color32::from_rgb(28, 41, 35));
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        dismiss = ui
                            .add(
                                egui::Button::new("×")
                                    .small()
                                    .fill(Color32::from_rgb(220, 231, 205)),
                            )
                            .on_hover_text("Nachricht schliessen")
                            .clicked();
                        let source = if self.error.is_some() {
                            "Hinweis"
                        } else {
                            self.message_source.as_deref().unwrap_or("Mini-Beni")
                        };
                        ui.add_sized(
                            [ui.available_width(), ui.spacing().interact_size.y],
                            egui::Label::new(egui::RichText::new(source).small())
                                .truncate()
                                .halign(egui::Align::Min),
                        );
                    });
                });
                egui::ScrollArea::vertical()
                    .id_salt("speech-text")
                    .max_height(125.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(egui::RichText::new(text).size(14.0)).wrap());
                    });
            })
            .response
            .interact(Sense::click());
        if dismiss || response.clicked() {
            self.dismiss_speech(now);
        }
        ui.add_space((78.0 - (ui.cursor().min.y - top)).max(0.0));
    }
    fn avatar_mesh(&self, stage: Rect) -> Result<egui::Mesh, buddy_core::BuddyError> {
        let pose = self.player.pose(&self.engine, &self.settings)?;
        let corners = self
            .engine
            .corners(self.player.state(), &pose, self.settings.mirrored)?;
        let [x, y, w, h, _, _] = self.engine.atlas.frames[&pose.frame];
        let [iw, ih] = self.engine.image_size.map(|n| n as f32);
        let uv = [
            egui::pos2(x as f32 / iw, y as f32 / ih),
            egui::pos2((x + w) as f32 / iw, y as f32 / ih),
            egui::pos2((x + w) as f32 / iw, (y + h) as f32 / ih),
            egui::pos2(x as f32 / iw, (y + h) as f32 / ih),
        ];
        let [cw, ch] = self.engine.atlas.window.map(|n| n as f32);
        let k = (stage.width() / cw).min(stage.height() / ch);
        let origin = stage.center() - egui::vec2(cw * k, ch * k) * 0.5;
        let mut mesh = egui::Mesh::with_texture(self.texture.id());
        for (corner, uv) in corners.into_iter().zip(uv) {
            mesh.vertices.push(egui::epaint::Vertex {
                pos: origin + egui::vec2(corner[0] as f32 * k, corner[1] as f32 * k),
                uv,
                color: Color32::WHITE,
            });
        }
        mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
        Ok(mesh)
    }
    fn paint_avatar(&mut self, ui: &mut egui::Ui, stage: Rect) {
        // Register interactions first: a failed draw must never remove the drag/click target.
        let response = ui.interact(stage, ui.id().with("avatar"), Sense::click_and_drag());
        if response.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if response.clicked() {
            self.play("hallo", "Hoi! Ich bin dein Desktop-Buddy.");
        }
        response.on_hover_text("Ziehen: verschieben · Klicken: Hallo sagen");
        match self.avatar_mesh(stage) {
            Ok(mesh) => {
                ui.painter().add(egui::Shape::mesh(mesh));
                self.last_render_error = None;
            }
            Err(error) => {
                let message = error.to_string();
                if self.last_render_error.as_ref() != Some(&message) {
                    self.report_error(message.clone(), Instant::now());
                    self.last_render_error = Some(message);
                }
                ui.painter().rect_filled(
                    stage.shrink(35.0),
                    24,
                    Color32::from_rgba_unmultiplied(28, 44, 38, 180),
                );
                ui.painter().text(
                    stage.center(),
                    egui::Align2::CENTER_CENTER,
                    "Beni",
                    FontId::proportional(28.0),
                    Color32::from_rgb(190, 235, 170),
                );
            }
        }
    }
    fn controls(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(Color32::from_rgba_unmultiplied(19, 29, 39, 235))
            .corner_radius(14)
            .inner_margin(8)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Hoi!").clicked() {
                        self.play("hallo", "Hoi! Schön bist du da.");
                    }
                    let label = if self.player.paused {
                        "Weiter"
                    } else {
                        "Pause"
                    };
                    if ui.button(label).clicked() {
                        self.player.paused = !self.player.paused;
                    }
                    if ui.button("Menü").clicked() {
                        self.menu_open = !self.menu_open;
                    }
                    if ui.button("Beenden").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
    }
    fn menu(&mut self, ctx: &egui::Context) {
        if !self.menu_open {
            return;
        }
        let mut open = true;
        egui::Window::new("Benis Einstellungen")
            .open(&mut open)
            .fixed_pos([12.0, 12.0])
            .default_width(296.0)
            .resizable(false)
            .max_height(360.0)
            .vscroll(true)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Welche Animation soll ich zeigen?");
                let states: Vec<_> = self
                    .engine
                    .animations
                    .iter()
                    .map(|(name, animation)| (name.clone(), animation.label.clone()))
                    .collect();
                egui::ScrollArea::vertical()
                    .max_height(70.0)
                    .show(ui, |ui| {
                        for (name, label) in states {
                            if ui
                                .selectable_label(self.player.state() == name, label.clone())
                                .clicked()
                            {
                                self.play(&name, &format!("Beni: {label}."));
                            }
                        }
                    });
                ui.separator();
                ui.add(egui::Slider::new(&mut self.settings.energy, 0.0..=1.0).text("Energie"));
                ui.add(egui::Slider::new(&mut self.settings.speed, 0.25..=3.0).text("Tempo"));
                ui.checkbox(&mut self.settings.reduced_motion, "Weniger Bewegung");
                ui.checkbox(&mut self.settings.mirrored, "Beni spiegeln");
                if ui
                    .checkbox(&mut self.settings.always_on_top, "Immer im Vordergrund")
                    .changed()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                        if self.settings.always_on_top {
                            egui::WindowLevel::AlwaysOnTop
                        } else {
                            egui::WindowLevel::Normal
                        },
                    ));
                }
                ui.separator();
                ui.checkbox(&mut self.ipc_enabled, "Statusmeldungen empfangen");
                ui.small("Lokale Programme können Beni Meldungen schicken.");
                ui.add(
                    egui::TextEdit::singleline(&mut self.speech_draft)
                        .hint_text("Benis Nachricht")
                        .char_limit(400),
                );
                if ui.button("Beni zeigt die Nachricht").clicked() {
                    let message = self.speech_draft.clone();
                    self.play("reden", &message);
                }
                ui.small("Esc schliesst das Menü. Alt/Option + Q beendet Beni.");
            });
        self.menu_open = open;
    }
}
impl BuddyApp {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let now = Instant::now();
        let delta = now.duration_since(self.last_tick).as_secs_f64() * 1000.0;
        self.last_tick = now;
        self.expire_error(now);
        if let Err(e) = self
            .player
            .advance(&self.engine, delta.min(1000.0), self.settings.speed)
        {
            self.report_error(e.to_string(), Instant::now());
            self.settings = self.settings.clone().sanitize();
        }
        let quit = ctx.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::Q));
        if quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.menu_open = false;
        }
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            self.speech(ui, now);
            let stage_height = (ui.available_height() - 52.0).max(100.0);
            let (_, stage) = ui.allocate_space(Vec2::new(ui.available_width(), stage_height));
            self.paint_avatar(ui, stage);
            self.controls(ui);
        });
        self.menu(&ctx);
        let occluded = ctx.input(|i| i.viewport().occluded.unwrap_or(false));
        if !occluded {
            // Fixed budget: idle 15 fps, active 30 fps, reduced motion / pause 4 fps.
            let fps = if self.settings.reduced_motion || self.player.paused {
                4
            } else if self.player.state() == "warten" {
                15
            } else {
                30
            };
            ctx.request_repaint_after(Duration::from_millis(1000 / fps));
        }
    }
}
impl BuddyApp {
    fn handle_tray(
        &mut self,
        action: Action,
        ctx: &egui::Context,
        window: Option<&winit::window::Window>,
    ) {
        match action {
            Action::Show | Action::Recover | Action::Settings => {
                self.hidden = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                if let Some(window) = window {
                    crate::window::restore(window, &mut self.placement, action == Action::Recover);
                }
                if action == Action::Settings {
                    self.menu_open = true;
                }
            }
            Action::Hide => {
                self.hidden = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
            Action::Pause => self.player.paused = !self.player.paused,
            Action::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }
}
impl eframe::App for BuddyApp {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let window = frame.winit_window().cloned();
        if let Some(window) = window.as_deref() {
            if !self.tray_attempted {
                self.tray_attempted = true;
                match Tray::new(
                    ctx.clone(),
                    self.tray_sender.clone(),
                    &self.engine,
                    self.player.paused,
                ) {
                    Ok(tray) => self.tray = Some(tray),
                    Err(error) => {
                        self.report_error(
                            format!("Menüleiste nicht verfügbar: {error}"),
                            Instant::now(),
                        );
                        crate::window::tray_unavailable(window);
                    }
                }
            }
            self.placement.check(window, false);
        }
        while let Ok(action) = self.tray_receiver.try_recv() {
            self.handle_tray(action, ctx, window.as_deref());
        }
        if window.is_some() && self.ipc_enabled && self.server.is_none() {
            let wake = ctx.clone();
            match buddy_ipc::Server::start(move || wake.request_repaint()) {
                Ok(server) => self.server = Some(server),
                Err(error) => {
                    self.ipc_enabled = false;
                    self.report_error(format!("Statusmeldungen: {error}"), Instant::now());
                }
            }
        }
        if !self.ipc_enabled {
            self.server = None;
        }
        if let Some(error) = self.server.as_ref().and_then(|s| s.take_error()) {
            self.server = None;
            self.ipc_enabled = false;
            self.report_error(format!("Statusmeldungen: {error}"), Instant::now());
        }
        let visible = !self.hidden
            && window.as_ref().is_none_or(|w| {
                w.is_visible().unwrap_or(true) && !w.is_minimized().unwrap_or(false)
            });
        let occluded = ctx.input(|i| i.viewport().occluded.unwrap_or(false));
        self.deliver_status(visible && !occluded, Instant::now());
        if let Some(tray) = &self.tray {
            tray.set_paused(self.player.paused);
        }
        self.expire_error(Instant::now());
        // Also check topology when hidden/occluded; avoid an animation loop in that state.
        ctx.request_repaint_after(Duration::from_secs(1));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show_ui(ui);
    }
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "buddy-settings", &self.settings);
        eframe::set_value(storage, "status-receiving", &self.ipc_enabled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod common {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../crates/buddy-ipc/tests/common/mod.rs"
        ));
    }
    fn app(ctx: &egui::Context) -> BuddyApp {
        let icon = eframe::icon_data::from_png_bytes(ATLAS_PNG).unwrap();
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [icon.width as usize, icon.height as usize],
            &icon.rgba,
        );
        let (tray_sender, tray_receiver) = sync_channel(32);
        BuddyApp {
            engine: Engine::embedded().unwrap(),
            player: Player::default(),
            texture: ctx.load_texture("test-atlas", image, egui::TextureOptions::LINEAR),
            settings: Settings::default(),
            last_tick: Instant::now(),
            message: "Hoi!".into(),
            message_until: Instant::now() + Duration::from_secs(10),
            message_source: None,
            active_remote: false,
            ipc_enabled: false,
            server: None,
            hidden: false,
            menu_open: false,
            speech_draft: "Hoi von den Tests!".into(),
            error: None,
            last_render_error: None,
            tray: None,
            tray_attempted: false,
            tray_sender,
            tray_receiver,
            placement: PlacementGuard::default(),
        }
    }
    fn frame(
        ctx: &egui::Context,
        app: &mut BuddyApp,
        time: f64,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(340.0, 440.0),
                )),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ui| app.show_ui(ui),
        )
    }
    fn find_text(shape: &egui::Shape, needle: &str) -> Option<Rect> {
        match shape {
            egui::Shape::Text(text) if text.galley.job.text == needle => {
                Some(text.galley.rect.translate(text.pos.to_vec2()))
            }
            egui::Shape::Vec(shapes) => shapes.iter().find_map(|s| find_text(s, needle)),
            _ => None,
        }
    }
    fn text_rect(output: &egui::FullOutput, needle: &str) -> Rect {
        output
            .shapes
            .iter()
            .find_map(|s| find_text(&s.shape, needle))
            .unwrap_or_else(|| panic!("UI text missing: {needle}"))
    }
    fn click(
        ctx: &egui::Context,
        app: &mut BuddyApp,
        pos: egui::Pos2,
        time: f64,
    ) -> egui::FullOutput {
        frame(
            ctx,
            app,
            time,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
        )
        .drop_without_applying_deltas();
        frame(
            ctx,
            app,
            time + 0.01,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        )
    }
    #[test]
    fn controls_remain_inside_native_window_after_multiple_frames() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        for i in 0..5 {
            let output = frame(&ctx, &mut app, i as f64 * 0.1, vec![]);
            for label in ["Hoi!", "Pause", "Menü", "Beenden"] {
                let rect = text_rect(&output, label);
                assert!(
                    rect.min.x >= 0.0
                        && rect.min.y >= 0.0
                        && rect.max.x <= 340.0
                        && rect.max.y <= 440.0,
                    "{label}: {rect:?}"
                );
            }
            output.drop_without_applying_deltas();
        }
        assert_eq!(
            eframe::App::clear_color(&app, &egui::Visuals::dark()),
            [0.0; 4]
        );
    }
    #[test]
    fn pointer_clicks_open_menu_change_motion_and_show_speech() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        let pos = text_rect(&output, "Menü").center();
        output.drop_without_applying_deltas();
        click(&ctx, &mut app, pos, 0.1).drop_without_applying_deltas();
        assert!(app.menu_open);
        let output = frame(&ctx, &mut app, 0.2, vec![]);
        let reduced = text_rect(&output, "Weniger Bewegung").center();
        output.drop_without_applying_deltas();
        click(&ctx, &mut app, reduced, 0.3).drop_without_applying_deltas();
        assert!(app.settings.reduced_motion);
        let output = frame(&ctx, &mut app, 0.4, vec![]);
        let send = text_rect(&output, "Beni zeigt die Nachricht").center();
        output.drop_without_applying_deltas();
        click(&ctx, &mut app, send, 0.5).drop_without_applying_deltas();
        assert_eq!(app.player.state(), "reden");
        assert_eq!(app.message, "Hoi von den Tests!");
    }
    #[test]
    fn dragging_avatar_requests_a_native_window_drag() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        let points = output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Mesh(mesh) if mesh.texture_id == app.texture.id() => {
                    Some(mesh.vertices.iter().map(|v| v.pos).collect::<Vec<_>>())
                }
                _ => None,
            })
            .expect("avatar mesh");
        let pos = Rect::from_points(&points).center();
        output.drop_without_applying_deltas();
        frame(
            &ctx,
            &mut app,
            0.1,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
        )
        .drop_without_applying_deltas();
        let output = frame(
            &ctx,
            &mut app,
            0.2,
            vec![egui::Event::PointerMoved(pos + egui::vec2(30.0, 0.0))],
        );
        assert!(
            output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .contains(&egui::ViewportCommand::StartDrag)
        );
        output.drop_without_applying_deltas();
    }
    #[test]
    fn pause_resume_and_quit_work_through_real_egui_responses() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        let pos = text_rect(&output, "Pause").center();
        output.drop_without_applying_deltas();
        click(&ctx, &mut app, pos, 0.1).drop_without_applying_deltas();
        assert!(app.player.paused);
        let output = frame(&ctx, &mut app, 0.2, vec![]);
        let pos = text_rect(&output, "Weiter").center();
        output.drop_without_applying_deltas();
        click(&ctx, &mut app, pos, 0.3).drop_without_applying_deltas();
        assert!(!app.player.paused);
        let output = frame(&ctx, &mut app, 0.4, vec![]);
        let pos = text_rect(&output, "Beenden").center();
        output.drop_without_applying_deltas();
        let output = click(&ctx, &mut app, pos, 0.5);
        assert!(
            output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .contains(&egui::ViewportCommand::Close)
        );
        output.drop_without_applying_deltas();
    }

    #[test]
    fn errors_expire_and_successful_actions_clear_them() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        let now = Instant::now();
        app.report_error("Testfehler".into(), now);
        app.expire_error(now + Duration::from_secs(5));
        assert!(app.error.is_some());
        app.expire_error(now + Duration::from_secs(6));
        assert!(app.error.is_none());
        app.play("unknown-state", "");
        assert!(app.error.is_some());
        app.play("hallo", "Hoi!");
        assert!(app.error.is_none());
    }

    #[test]
    fn rendering_failure_keeps_drag_menu_and_quit_available() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        // Remove the idle pose to exercise the actual fallible rendering path.
        let pose = app.player.pose(&app.engine, &app.settings).unwrap();
        app.engine.atlas.frames.remove(&pose.frame);
        app.player.paused = true;
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        let pos = text_rect(&output, "Beni").center();
        let deadline = app.error.as_ref().unwrap().until;
        output.drop_without_applying_deltas();
        frame(
            &ctx,
            &mut app,
            0.1,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
        )
        .drop_without_applying_deltas();
        let output = frame(
            &ctx,
            &mut app,
            0.2,
            vec![egui::Event::PointerMoved(pos + egui::vec2(30.0, 0.0))],
        );
        assert!(
            output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .contains(&egui::ViewportCommand::StartDrag)
        );
        assert_eq!(app.error.as_ref().unwrap().until, deadline);
        output.drop_without_applying_deltas();
        frame(
            &ctx,
            &mut app,
            0.3,
            vec![egui::Event::PointerButton {
                pos: pos + egui::vec2(30.0, 0.0),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        )
        .drop_without_applying_deltas();
        app.expire_error(deadline);
        let output = frame(&ctx, &mut app, 0.4, vec![]);
        assert!(
            app.error.is_none(),
            "persistent draw errors must not restart the timer"
        );
        let menu = text_rect(&output, "Menü").center();
        output.drop_without_applying_deltas();
        click(&ctx, &mut app, menu, 0.5).drop_without_applying_deltas();
        assert!(app.menu_open);
        app.menu_open = false;
        let output = frame(&ctx, &mut app, 0.6, vec![]);
        let quit = text_rect(&output, "Beenden").center();
        output.drop_without_applying_deltas();
        let output = click(&ctx, &mut app, quit, 0.7);
        assert!(
            output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .contains(&egui::ViewportCommand::Close)
        );
        output.drop_without_applying_deltas();
    }

    #[test]
    fn tray_actions_work_without_a_render_pass() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        let mut native_frame = eframe::Frame::_new_kittest();
        for action in [
            Action::Hide,
            Action::Show,
            Action::Recover,
            Action::Pause,
            Action::Settings,
            Action::Quit,
        ] {
            app.tray_sender.try_send(action).unwrap();
            let output = ctx.run_logic(&egui::RawInput::default(), |ctx| {
                eframe::App::logic(&mut app, ctx, &mut native_frame);
            });
            match action {
                Action::Show | Action::Recover | Action::Settings => {
                    let commands = &output.viewport_commands[&egui::ViewportId::ROOT];
                    for command in [
                        egui::ViewportCommand::Visible(true),
                        egui::ViewportCommand::Minimized(false),
                        egui::ViewportCommand::Focus,
                    ] {
                        assert!(commands.contains(&command));
                    }
                    if action == Action::Settings {
                        assert!(app.menu_open);
                    }
                }
                Action::Hide => assert!(
                    output.viewport_commands[&egui::ViewportId::ROOT]
                        .contains(&egui::ViewportCommand::Visible(false))
                ),
                Action::Quit => assert!(
                    output.viewport_commands[&egui::ViewportId::ROOT]
                        .contains(&egui::ViewportCommand::Close)
                ),
                Action::Pause => assert!(app.player.paused),
            }
        }
        app.tray_sender.try_send(Action::Pause).unwrap();
        let _ = ctx.run_logic(&egui::RawInput::default(), |ctx| {
            eframe::App::logic(&mut app, ctx, &mut native_frame);
        });
        assert!(!app.player.paused);
    }

    #[test]
    fn long_speech_scrolls_and_can_be_closed_without_hiding_controls() {
        let ctx = egui::Context::default();
        ctx.all_styles_mut(|style| style.animation_time = 0.0);
        let mut app = app(&ctx);
        let text = "Beni erklärt den nächsten Schritt. ".repeat(11);
        app.play("reden", &text);
        app.message_source = Some("W".repeat(48));
        assert!(app.message_until.duration_since(Instant::now()) > Duration::from_secs(20));
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        for label in ["Hoi!", "Pause", "Menü", "Beenden", "×"] {
            let rect = text_rect(&output, label);
            assert!(
                rect.min.y >= 0.0 && rect.max.y <= 440.0,
                "{label}: {rect:?}"
            );
        }
        let (text_rect_before, clip) = output
            .shapes
            .iter()
            .find_map(|s| find_text(&s.shape, &text).map(|r| (r, s.clip_rect)))
            .unwrap();
        assert!(
            text_rect_before.max.y > clip.max.y,
            "long text must have a scroll range"
        );
        let pos = clip.center();
        output.drop_without_applying_deltas();
        frame(
            &ctx,
            &mut app,
            0.1,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::MouseWheel {
                    phase: egui::TouchPhase::Move,
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -2000.0),
                    modifiers: Default::default(),
                },
            ],
        )
        .drop_without_applying_deltas();
        let output = frame(&ctx, &mut app, 0.2, vec![]);
        let (after, clip) = output
            .shapes
            .iter()
            .find_map(|s| find_text(&s.shape, &text).map(|r| (r, s.clip_rect)))
            .unwrap();
        assert!(
            after.max.y <= clip.max.y + 1.0,
            "last line must be reachable: {after:?} / {clip:?}"
        );
        let close = text_rect(&output, "×").center();
        output.drop_without_applying_deltas();
        click(&ctx, &mut app, close, 0.3).drop_without_applying_deltas();
        assert!(app.message_until <= Instant::now());
    }

    #[test]
    fn received_status_waits_while_hidden_and_then_shows_text_source_and_gesture() {
        let endpoint = common::TestEndpoint::new();
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        assert!(!app.ipc_enabled, "receiving is opt-in");
        let wake = ctx.clone();
        app.server = Some(
            buddy_ipc::Server::start_at(endpoint.endpoint.clone(), move || wake.request_repaint())
                .unwrap(),
        );
        app.ipc_enabled = true;
        app.hidden = true;
        let mut message = buddy_core::notifications::Notification::new("Build grün");
        message.source = "test-build".into();
        buddy_ipc::notify(&endpoint.endpoint, message).unwrap();
        let mut native_frame = eframe::Frame::_new_kittest();
        let _ = ctx.run_logic(&egui::RawInput::default(), |ctx| {
            eframe::App::logic(&mut app, ctx, &mut native_frame)
        });
        assert_eq!(app.server.as_ref().unwrap().pending(), 1);
        app.hidden = false;
        let _ = ctx.run_logic(&egui::RawInput::default(), |ctx| {
            eframe::App::logic(&mut app, ctx, &mut native_frame)
        });
        assert_eq!(app.player.state(), "daumen");
        assert_eq!(app.message, "Build grün");
        assert_eq!(app.message_source.as_deref(), Some("test-build"));
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        text_rect(&output, "Build grün");
        text_rect(&output, "test-build");
        output.drop_without_applying_deltas();
        app.ipc_enabled = false;
        let _ = ctx.run_logic(&egui::RawInput::default(), |ctx| {
            eframe::App::logic(&mut app, ctx, &mut native_frame)
        });
        assert!(app.server.is_none());
        assert!(
            buddy_ipc::notify(
                &endpoint.endpoint,
                buddy_core::notifications::Notification::new("ok")
            )
            .is_err()
        );
    }
}
