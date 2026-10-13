use crate::{
    size::BuddySize,
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
    atlas_alpha: Vec<u8>,
    hit_map: crate::hit_map::HitMap,
    input: crate::input::InputGuard,
    click_through: bool,
    size: BuddySize,
    size_pending: bool,
    hotkeys: crate::hotkeys::Hotkeys,
    history: crate::history::History,
    history_player: Option<Player>,
    settings: Settings,
    last_tick: Instant,
    message: String,
    message_until: Instant,
    message_source: Option<String>,
    active_remote: bool,
    do_not_disturb: bool,
    ipc_enabled: bool,
    server: Option<buddy_ipc::Server>,
    hidden: bool,
    controls_open: bool,
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
        let texture =
            cc.egui_ctx
                .load_texture("beni-atlas", image.clone(), egui::TextureOptions::LINEAR);
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
            atlas_alpha: image.pixels.iter().map(|pixel| pixel.a()).collect(),
            hit_map: crate::hit_map::HitMap::default(),
            input: crate::input::InputGuard::default(),
            click_through: cc.storage.and_then(|s| eframe::get_value(s, "click-through")).unwrap_or(true),
            settings,
            size: BuddySize::load(cc.storage),
            size_pending: true,
            hotkeys: crate::hotkeys::Hotkeys::load(cc.storage),
            history: crate::history::History::default(),
            history_player: None,
            last_tick: Instant::now(),
            message: "Hoi! Ich bin Beni. Klick mich für die Bedienung an oder zieh mich an deinen Lieblingsplatz.".into(),
            message_until: Instant::now() + Duration::from_secs(10),
            message_source: None,
            active_remote: false,
            do_not_disturb: Self::load_do_not_disturb(cc.storage),
            ipc_enabled: cc
                .storage
                .and_then(|s| eframe::get_value(s, "status-receiving"))
                .unwrap_or(false),
            server: None,
            hidden: false,
            controls_open: false,
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
        self.close_history();
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
    fn repaint_delay(&self, now: Instant, visible: bool) -> Duration {
        // Keep the once-per-second topology check, history ages and reduced-motion
        // one-shots progressing. Input, IPC and tray actions wake the UI directly.
        let housekeeping = Duration::from_secs(1);
        if !visible {
            return housekeeping;
        }
        let mut delay = if self.settings.reduced_motion || self.player.paused {
            housekeeping
        } else if self.player.state() == "warten" {
            Duration::from_millis(1000 / 15)
        } else {
            Duration::from_millis(1000 / 30)
        };
        // Static poses still need to remove transient text at its deadline.
        if self.history_player.is_none() {
            let deadline = self.error.as_ref().map_or(self.message_until, |e| e.until);
            if deadline > now {
                delay = delay.min(deadline.duration_since(now));
            }
        }
        delay
    }
    fn deliver_status(&mut self, visible: bool, now: Instant) {
        if self.do_not_disturb
            || !visible
            || self.menu_open
            || self.history_player.is_some()
            || self.error.is_some()
            || (self.active_remote && now < self.message_until)
        {
            return;
        }
        if let Some(delivery) = self.server.as_ref().and_then(|s| s.pop()) {
            let message = delivery.message;
            let paused = self.player.paused;
            self.play(message.state.as_deref().unwrap_or("reden"), &message.text);
            self.player.paused = paused;
            self.message_source = Some(message.source);
            self.message_until = now
                + Duration::from_millis(reading_time_ms(&message.text).min(delivery.remaining_ms));
            self.active_remote = true;
        }
    }
    fn collect_history(&mut self) {
        if let Some(server) = &self.server {
            self.history.extend(server.drain_received());
        }
    }
    fn stop_receiving(&mut self) {
        if let Some(server) = self.server.take() {
            self.history.extend(server.stop_and_drain());
        }
    }
    fn open_history(&mut self) {
        self.collect_history();
        self.menu_open = false;
        self.controls_open = true;
        if self.history_player.is_some() {
            return;
        }
        let previous = self.player.clone();
        if let Err(error) = self.player.play(&self.engine, "denken", false) {
            self.report_error(error.to_string(), Instant::now());
            return;
        }
        self.player.paused = previous.paused;
        self.history_player = Some(previous);
    }
    fn close_history(&mut self) {
        if let Some(mut previous) = self.history_player.take() {
            previous.paused = self.player.paused;
            self.player = previous;
        }
    }
    fn history_ui(&mut self, ui: &mut egui::Ui, now: Instant) {
        let (rect, action) = self.history.ui(ui, now);
        self.hit_map.add_rect(rect, 14.0);
        if action.clear {
            self.collect_history();
            self.history.clear();
        }
        if action.close {
            self.close_history();
        }
    }
    fn load_do_not_disturb(storage: Option<&dyn eframe::Storage>) -> bool {
        storage
            .and_then(|s| eframe::get_value(s, "do-not-disturb"))
            .unwrap_or(false)
    }
    fn set_do_not_disturb(&mut self, enabled: bool, now: Instant) {
        self.do_not_disturb = enabled;
        if enabled && self.active_remote {
            self.message_until = now;
            self.message_source = None;
            self.active_remote = false;
            // Keep animation pause and errors independent of status delivery.
            let player = self.history_player.as_mut().unwrap_or(&mut self.player);
            let paused = player.paused;
            let _ = player.play(&self.engine, "warten", true);
            player.paused = paused;
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
                        let close = ui
                            .add(
                                egui::Button::new("×")
                                    .small()
                                    .fill(Color32::from_rgb(220, 231, 205)),
                            )
                            .on_hover_text("Nachricht schliessen");
                        close.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::Button,
                                close.enabled(),
                                "Nachricht schliessen",
                            )
                        });
                        dismiss = close.clicked();
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
            // The labelled close button provides keyboard and screenreader access.
            // Clicking the background remains a pointer-only shortcut.
            .interact(Sense::CLICK);
        if dismiss || response.clicked() {
            self.dismiss_speech(now);
        }
        self.hit_map.add_rect(response.rect, 16.0);
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
            self.controls_open = !self.controls_open;
            if !self.controls_open {
                self.menu_open = false;
                self.close_history();
            }
        }
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Button,
                response.enabled(),
                if self.controls_open {
                    "Beni: Bedienleiste zuklappen"
                } else {
                    "Beni: Bedienleiste aufklappen"
                },
            )
        });
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_expanded(self.controls_open);
            if let Some(animation) = self.engine.animations.get(self.player.state()) {
                node.set_description(format!("Animation: {}", animation.label));
            }
        });
        response.on_hover_text(if self.controls_open {
            "Ziehen: verschieben · Klicken: Bedienung zuklappen"
        } else {
            "Ziehen: verschieben · Klicken: Bedienung aufklappen"
        });
        match self.avatar_mesh(stage) {
            Ok(mesh) => {
                self.hit_map
                    .add_mesh(&mesh, &self.atlas_alpha, self.engine.image_size[0] as usize);
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
                self.hit_map.add_rect(stage.shrink(35.0), 24.0);
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
        let response = egui::Frame::new()
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
                        self.close_history();
                        self.menu_open = !self.menu_open;
                    }
                    if ui.button("Beenden").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            })
            .response;
        self.hit_map.add_rect(response.rect, 14.0);
    }
    fn menu(&mut self, ctx: &egui::Context) {
        if !self.menu_open {
            return;
        }
        let mut open = true;
        let available = ctx.content_rect().size() - Vec2::splat(24.0);
        let response = egui::Window::new("Benis Einstellungen")
            .open(&mut open)
            .fixed_pos([12.0, 12.0])
            .fixed_size([available.x, available.y.min(360.0)])
            .vscroll(true)
            .collapsible(false)
            .show(ctx, |ui| {
                if ui.button("Verlauf öffnen").clicked() {
                    self.open_history();
                }
                ui.separator();
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
                ui.label("Beni-Grösse");
                ui.horizontal(|ui| {
                    for size in BuddySize::ALL {
                        if ui.radio_value(&mut self.size, size, size.label()).changed() {
                            self.size_pending = true;
                        }
                    }
                });
                ui.add(egui::Slider::new(&mut self.settings.energy, 0.0..=1.0).text("Energie"));
                ui.add(egui::Slider::new(&mut self.settings.speed, 0.25..=3.0).text("Tempo"));
                ui.checkbox(&mut self.settings.reduced_motion, "Weniger Bewegung");
                ui.checkbox(&mut self.settings.mirrored, "Beni spiegeln");
                if ui
                    .checkbox(&mut self.click_through, "Hintergrund durchklickbar")
                    .changed()
                    && self.click_through
                {
                    self.input.retry_if_failed();
                }
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
                let mut quiet = self.do_not_disturb;
                if ui.checkbox(&mut quiet, "Nicht stören").changed() {
                    self.set_do_not_disturb(quiet, Instant::now());
                }
                ui.small("Pausiert Statusanzeigen. Meldungen verfallen weiterhin.");
                let message = ui.add(
                    egui::TextEdit::singleline(&mut self.speech_draft)
                        .hint_text("Benis Nachricht")
                        .char_limit(400),
                );
                ctx.accesskit_node_builder(message.id, |node| {
                    node.set_label("Benis Nachricht");
                });
                if ui.button("Beni zeigt die Nachricht").clicked() {
                    let message = self.speech_draft.clone();
                    self.play("reden", &message);
                }
                self.hotkeys.ui(ui);
                ui.small("Esc klappt die Bedienung zu. Alt/Option + Q beendet Beni.");
            });
        if let Some(response) = response {
            self.hit_map.add_rect(response.response.rect, 0.0);
        }
        self.menu_open = open && self.history_player.is_none();
    }
}
impl BuddyApp {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.hit_map.reset(ctx.content_rect().size());
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
            self.controls_open = false;
            self.close_history();
        }
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            if self.history_player.is_none() {
                self.speech(ui, now);
            }
            if self.do_not_disturb {
                let hint = ui.small("Nicht stören ist aktiv");
                self.hit_map.add_rect(hint.rect, 0.0);
            }
            // Keep Beni's size and position stable when the controls toggle.
            let history_height = if self.history_player.is_some() {
                180.0
            } else {
                0.0
            };
            let stage_height = (ui.available_height() - 52.0 - history_height).max(100.0);
            let (_, stage) = ui.allocate_space(Vec2::new(ui.available_width(), stage_height));
            self.paint_avatar(ui, stage);
            if self.history_player.is_some() {
                self.history_ui(ui, now);
            }
            if self.controls_open {
                self.controls(ui);
            }
        });
        self.menu(&ctx);
        if self.size_pending {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(self.size.dimensions()));
            self.size_pending = false;
        }
        let occluded = ctx.input(|i| i.viewport().occluded.unwrap_or(false));
        ctx.request_repaint_after(self.repaint_delay(now, !self.hidden && !occluded));
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
            Action::Show | Action::Recover | Action::Settings | Action::History => {
                self.hidden = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                if let Some(window) = window {
                    crate::window::restore(window, &mut self.placement, action == Action::Recover);
                }
                if action == Action::Settings {
                    self.close_history();
                    self.menu_open = true;
                } else if action == Action::History {
                    self.open_history();
                }
            }
            Action::Hide => {
                self.hidden = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
            Action::Pause => self.player.paused = !self.player.paused,
            Action::DoNotDisturb => {
                self.set_do_not_disturb(!self.do_not_disturb, Instant::now());
            }
            Action::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }
}
impl eframe::App for BuddyApp {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let window = frame.winit_window().cloned();
        if let Some(window) = window.as_deref() {
            self.hotkeys.sync(ctx, window);
            if !self.tray_attempted {
                self.tray_attempted = true;
                match Tray::new(
                    ctx.clone(),
                    self.tray_sender.clone(),
                    &self.engine,
                    self.player.paused,
                    self.do_not_disturb,
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
        while let Some(action) = self.hotkeys.pop() {
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
        self.collect_history();
        if !self.ipc_enabled {
            self.stop_receiving();
        }
        if let Some(error) = self.server.as_ref().and_then(|s| s.take_error()) {
            self.stop_receiving();
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
            tray.set_do_not_disturb(self.do_not_disturb);
        }
        self.expire_error(Instant::now());
        // Also check topology when hidden/occluded; avoid an animation loop in that state.
        ctx.request_repaint_after(Duration::from_secs(1));
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.show_ui(ui);
        if let Some(window) = frame.winit_window() {
            let enabled = self.click_through && self.tray.is_some() && !window.is_decorated();
            if let Err(message) = self.input.apply(window, &self.hit_map, ui.ctx(), enabled) {
                self.click_through = false;
                self.report_error(message.into(), Instant::now());
            }
        }
    }
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "buddy-settings", &self.settings);
        eframe::set_value(storage, "status-receiving", &self.ipc_enabled);
        eframe::set_value(storage, "click-through", &self.click_through);
        eframe::set_value(storage, "do-not-disturb", &self.do_not_disturb);
        self.size.save(storage);
        self.hotkeys.save(storage);
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
            atlas_alpha: icon
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .map(|pixel| pixel[3])
                .collect(),
            hit_map: crate::hit_map::HitMap::default(),
            input: crate::input::InputGuard::default(),
            click_through: true,
            size: BuddySize::default(),
            size_pending: true,
            hotkeys: crate::hotkeys::Hotkeys::load(None),
            history: crate::history::History::default(),
            history_player: None,
            settings: Settings::default(),
            last_tick: Instant::now(),
            message: "Hoi aus der Test-Sprechblase!".into(),
            message_until: Instant::now() + Duration::from_secs(10),
            message_source: None,
            active_remote: false,
            do_not_disturb: false,
            ipc_enabled: false,
            server: None,
            hidden: false,
            controls_open: false,
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
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, app.size.dimensions())),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ui| app.show_ui(ui),
        )
    }
    #[test]
    fn settled_ui_sleeps_for_static_poses_and_keeps_animated_budgets() {
        for (reduced, paused, state, expected) in [
            (true, false, "warten", Duration::from_secs(1)),
            (false, true, "denken", Duration::from_secs(1)),
            (false, false, "warten", Duration::from_millis(66)),
            (false, false, "denken", Duration::from_millis(33)),
        ] {
            let ctx = egui::Context::default();
            ctx.all_styles_mut(|style| style.animation_time = 0.0);
            let mut app = app(&ctx);
            app.settings.reduced_motion = reduced;
            app.player.play(&app.engine, state, false).unwrap();
            app.player.paused = paused;
            app.message_until = Instant::now();
            let mut delay = Duration::ZERO;
            for index in 0..6 {
                let mut output = frame(&ctx, &mut app, index as f64 * 0.02, vec![]);
                delay = output.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
                output.textures_delta.clear();
            }
            assert!(
                delay <= expected && delay >= expected.saturating_sub(Duration::from_millis(20)),
                "reduced={reduced}, paused={paused}, state={state}: {delay:?}"
            );
        }
    }
    #[test]
    fn static_repaints_honour_speech_and_error_deadlines_without_spinning() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        let now = Instant::now();
        app.player.paused = true;
        app.message_until = now + Duration::from_millis(120);
        assert_eq!(app.repaint_delay(now, true), Duration::from_millis(120));
        app.report_error("Hinweis".into(), now);
        // The error hides the message; its six-second lifetime needs no fast loop.
        assert_eq!(app.repaint_delay(now, true), Duration::from_secs(1));
        assert_eq!(
            app.repaint_delay(now + Duration::from_millis(5900), true),
            Duration::from_millis(100)
        );
        app.expire_error(now + Duration::from_secs(6));
        assert_eq!(
            app.repaint_delay(now + Duration::from_secs(6), true),
            Duration::from_secs(1)
        );
        app.open_history();
        assert_eq!(app.repaint_delay(now, true), Duration::from_secs(1));
        app.close_history();
        app.player.paused = false;
        // Hidden/occluded windows never inherit active-gesture or speech deadlines.
        app.player.play(&app.engine, "denken", false).unwrap();
        assert_eq!(app.repaint_delay(now, false), Duration::from_secs(1));
    }
    #[test]
    fn reduced_motion_one_shot_still_returns_to_idle_and_expires_speech() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.settings.reduced_motion = true;
        app.play("daumen", "Fertig");
        app.message_until = Instant::now() - Duration::from_secs(1);
        for index in 0..10 {
            app.last_tick = Instant::now() - Duration::from_secs(1);
            let mut output = frame(&ctx, &mut app, index as f64, vec![]);
            let has_speech = output
                .shapes
                .iter()
                .any(|s| find_text(&s.shape, "Fertig").is_some());
            output.textures_delta.clear();
            assert!(!has_speech);
        }
        assert_eq!(app.player.state(), "warten");
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
    fn avatar_rect(output: &egui::FullOutput, app: &BuddyApp) -> Rect {
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
        Rect::from_points(&points)
    }
    fn assert_controls_hidden(output: &egui::FullOutput) {
        for label in ["Hoi!", "Pause", "Weiter", "Menü", "Beenden"] {
            assert!(
                !output
                    .shapes
                    .iter()
                    .any(|s| find_text(&s.shape, label).is_some()),
                "collapsed controls must not show {label}"
            );
        }
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
    fn accesskit_node<'a>(
        output: &'a egui::FullOutput,
        label: &str,
    ) -> (egui::accesskit::NodeId, &'a egui::accesskit::Node) {
        let update = output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("accessibility tree");
        update
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .map(|(id, node)| (*id, node))
            .unwrap_or_else(|| panic!("accessible node missing: {label}"))
    }
    fn accessible_click(target: egui::accesskit::NodeId) -> egui::Event {
        egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target_tree: egui::accesskit::TreeId::ROOT,
            target_node: target,
            data: None,
        })
    }
    #[derive(Default)]
    struct TestStorage(std::collections::HashMap<String, String>);
    impl eframe::Storage for TestStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
        fn set_string(&mut self, key: &str, value: String) {
            self.0.insert(key.into(), value);
        }
        fn remove_string(&mut self, key: &str) {
            self.0.remove(key);
        }
        fn flush(&mut self) {}
    }
    #[test]
    fn hotkey_actions_work_while_hidden_without_a_render_pass() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.hidden = true;
        let mut native_frame = eframe::Frame::_new_kittest();
        app.hotkeys.test_queue(Action::Pause);
        let _ = ctx.run_logic(&egui::RawInput::default(), |ctx| {
            eframe::App::logic(&mut app, ctx, &mut native_frame);
        });
        assert!(app.player.paused && app.hidden);
        app.hotkeys.test_queue(Action::Recover);
        let output = ctx.run_logic(&egui::RawInput::default(), |ctx| {
            eframe::App::logic(&mut app, ctx, &mut native_frame);
        });
        assert!(!app.hidden);
        for command in [
            egui::ViewportCommand::Visible(true),
            egui::ViewportCommand::Minimized(false),
            egui::ViewportCommand::Focus,
        ] {
            assert!(output.viewport_commands[&egui::ViewportId::ROOT].contains(&command));
        }
        app.hotkeys.test_queue(Action::Pause);
        let _ = ctx.run_logic(&egui::RawInput::default(), |ctx| {
            eframe::App::logic(&mut app, ctx, &mut native_frame);
        });
        assert!(!app.player.paused);
    }
    #[test]
    fn hotkey_preferences_are_opt_in_and_invalid_stored_values_disable_them() {
        let mut storage = TestStorage::default();
        let hotkeys = crate::hotkeys::Hotkeys::load(None);
        hotkeys.save(&mut storage);
        assert!(
            !eframe::get_value::<(bool, String, String)>(&storage, "global-hotkeys")
                .unwrap()
                .0
        );
        for value in [
            "(true,\"Ctrl+Alt+R\",\"Ctrl+Alt+S\")",
            "(false,\"F8\",\"F9\")",
        ] {
            eframe::Storage::set_string(&mut storage, "global-hotkeys", value.into());
            crate::hotkeys::Hotkeys::load(Some(&storage)).save(&mut storage);
            assert_eq!(
                eframe::Storage::get_string(&storage, "global-hotkeys").unwrap(),
                value
            );
        }
        for value in [
            "broken",
            "(true,\"B\",\"P\")",
            "(true,\"Ctrl+B\",\"Ctrl+B\")",
            "(true,\"Ctrl+AudioVolumeMute\",\"F8\")",
        ] {
            eframe::Storage::set_string(&mut storage, "global-hotkeys", value.into());
            crate::hotkeys::Hotkeys::load(Some(&storage)).save(&mut storage);
            assert!(
                !eframe::get_value::<(bool, String, String)>(&storage, "global-hotkeys")
                    .unwrap()
                    .0
            );
        }
    }
    #[test]
    fn size_persistence_preserves_settings_and_rejects_invalid_values() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        let mut storage = TestStorage::default();
        assert_eq!(BuddySize::load(None), BuddySize::Medium);
        assert_eq!(BuddySize::load(Some(&storage)), BuddySize::Medium);
        app.settings.energy = 0.45;
        app.ipc_enabled = true;
        for size in BuddySize::ALL {
            app.size = size;
            eframe::App::save(&mut app, &mut storage);
            assert_eq!(BuddySize::load(Some(&storage)), size);
            assert_eq!(
                eframe::get_value::<Settings>(&storage, "buddy-settings")
                    .unwrap()
                    .energy,
                0.45
            );
            assert_eq!(
                eframe::get_value::<bool>(&storage, "status-receiving"),
                Some(true)
            );
        }
        for value in ["\"huge\"", "\"SMALL\"", "123", "not-valid-ron", "\"NaN\""] {
            eframe::Storage::set_string(&mut storage, "buddy-size", value.into());
            assert_eq!(BuddySize::load(Some(&storage)), BuddySize::Medium);
        }
    }
    #[test]
    fn size_selection_resizes_once_without_replacing_a_status() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        ctx.all_styles_mut(|style| style.animation_time = 0.0);
        let mut app = app(&ctx);
        app.play("daumen", "Build grün");
        app.message_source = Some("codex-test".into());
        app.active_remote = true;
        app.player.paused = true;
        app.menu_open = true;
        let until = app.message_until;
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        assert!(
            output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .contains(&egui::ViewportCommand::InnerSize(
                    BuddySize::Medium.dimensions()
                ))
        );
        output.drop_without_applying_deltas();
        for (i, size) in [BuddySize::Small, BuddySize::Large, BuddySize::Medium]
            .into_iter()
            .enumerate()
        {
            let t = 1.0 + i as f64;
            let output = frame(&ctx, &mut app, t, vec![]);
            let target = text_rect(&output, size.label()).center();
            output.drop_without_applying_deltas();
            let output = click(&ctx, &mut app, target, t + 0.1);
            assert_eq!(app.size, size);
            assert!(
                output.viewport_output[&egui::ViewportId::ROOT]
                    .commands
                    .contains(&egui::ViewportCommand::InnerSize(size.dimensions()))
            );
            output.drop_without_applying_deltas();
            let output = frame(&ctx, &mut app, t + 0.2, vec![]);
            assert_eq!(
                accesskit_node(&output, size.label()).1.toggled(),
                Some(egui::accesskit::Toggled::True)
            );
            assert!(
                !output.viewport_output[&egui::ViewportId::ROOT]
                    .commands
                    .iter()
                    .any(|cmd| matches!(cmd, egui::ViewportCommand::InnerSize(_)))
            );
            assert!(app.menu_open);
            assert_eq!(app.message, "Build grün");
            assert_eq!(app.message_source.as_deref(), Some("codex-test"));
            assert_eq!(app.message_until, until);
            assert_eq!(app.player.state(), "daumen");
            assert!(app.active_remote && app.player.paused);
            output.drop_without_applying_deltas();
        }
    }
    #[test]
    fn all_sizes_keep_controls_settings_and_input_regions_reachable() {
        let mut previous_area = 0.0;
        for size in BuddySize::ALL {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = app(&ctx);
            app.size = size;
            app.player.paused = true;
            app.controls_open = true;
            let output = frame(&ctx, &mut app, 0.0, vec![]);
            let avatar = avatar_rect(&output, &app);
            let screen = Rect::from_min_size(egui::Pos2::ZERO, size.dimensions());
            assert!(screen.contains_rect(avatar));
            assert!(avatar.area() > previous_area);
            previous_area = avatar.area();
            assert!(app.hit_map.contains(avatar.center()));
            assert!(!app.hit_map.contains(egui::pos2(2.0, 2.0)));
            for label in ["Hoi!", "Weiter", "Menü", "Beenden", "×"] {
                let rect = text_rect(&output, label);
                assert!(screen.contains_rect(rect), "{size:?}: {label} {rect:?}");
                assert!(app.hit_map.contains(rect.center()));
            }
            output.drop_without_applying_deltas();
            app.menu_open = true;
            frame(&ctx, &mut app, 0.1, vec![]).drop_without_applying_deltas();
            let output = frame(&ctx, &mut app, 0.2, vec![]);
            let bounds = accesskit_node(&output, "Benis Einstellungen")
                .1
                .bounds()
                .unwrap();
            assert!(bounds.x0 >= 0.0 && bounds.y0 >= 0.0);
            assert!(
                bounds.x1 <= f64::from(screen.max.x) && bounds.y1 <= f64::from(screen.max.y),
                "{size:?}: {bounds:?}"
            );
            for choice in BuddySize::ALL {
                let rect = text_rect(&output, choice.label());
                assert!(screen.contains_rect(rect));
                assert!(app.hit_map.contains(rect.center()));
            }
            output.drop_without_applying_deltas();
        }
    }
    #[test]
    fn accessible_avatar_and_close_button_have_names_and_work_without_a_pointer() {
        use egui::accesskit::{Action, Role};
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = app(&ctx);
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        let (avatar, node) = accesskit_node(&output, "Beni: Bedienleiste aufklappen");
        assert_eq!(node.role(), Role::Button);
        assert_eq!(node.is_expanded(), Some(false));
        assert!(node.supports_action(Action::Focus));
        assert!(node.supports_action(Action::Click));
        let (close, node) = accesskit_node(&output, "Nachricht schliessen");
        assert_eq!(node.role(), Role::Button);
        assert!(node.supports_action(Action::Click));
        assert_controls_hidden(&output);
        output.drop_without_applying_deltas();

        let output = frame(&ctx, &mut app, 0.1, vec![accessible_click(avatar)]);
        assert!(app.controls_open);
        let (_, node) = accesskit_node(&output, "Beni: Bedienleiste zuklappen");
        assert_eq!(node.is_expanded(), Some(true));
        for label in ["Hoi!", "Pause", "Menü", "Beenden"] {
            assert_eq!(accesskit_node(&output, label).1.role(), Role::Button);
        }
        output.drop_without_applying_deltas();

        frame(&ctx, &mut app, 0.2, vec![accessible_click(close)]).drop_without_applying_deltas();
        assert!(app.message_until <= Instant::now());
        let output = frame(&ctx, &mut app, 0.3, vec![accessible_click(avatar)]);
        assert!(!app.controls_open);
        assert_eq!(
            accesskit_node(&output, "Beni: Bedienleiste aufklappen")
                .1
                .is_expanded(),
            Some(false)
        );
        assert_controls_hidden(&output);
        output.drop_without_applying_deltas();
    }
    #[test]
    fn tab_enter_and_space_operate_the_collapsed_avatar_without_dragging() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = app(&ctx);
        app.dismiss_speech(Instant::now());
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        let avatar = accesskit_node(&output, "Beni: Bedienleiste aufklappen").0;
        output.drop_without_applying_deltas();
        for (i, key) in [egui::Key::Tab, egui::Key::Enter, egui::Key::Space]
            .into_iter()
            .enumerate()
        {
            let time = 0.1 + i as f64 * 0.1;
            let output = frame(
                &ctx,
                &mut app,
                time,
                vec![egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                }],
            );
            assert_eq!(
                output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .focus,
                avatar
            );
            assert_eq!(app.controls_open, key == egui::Key::Enter);
            assert!(
                !output.viewport_output[&egui::ViewportId::ROOT]
                    .commands
                    .contains(&egui::ViewportCommand::StartDrag)
            );
            output.drop_without_applying_deltas();
            frame(
                &ctx,
                &mut app,
                time + 0.01,
                vec![egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Default::default(),
                }],
            )
            .drop_without_applying_deltas();
        }
    }
    #[test]
    fn accessible_settings_expose_named_controls_and_message_input() {
        use egui::accesskit::Role;
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = app(&ctx);
        app.menu_open = true;
        // egui sizes the settings window on its initial pass.
        frame(&ctx, &mut app, 0.0, vec![]).drop_without_applying_deltas();
        let output = frame(&ctx, &mut app, 0.1, vec![]);
        for label in [
            "Weniger Bewegung",
            "Beni spiegeln",
            "Statusmeldungen empfangen",
        ] {
            assert_eq!(accesskit_node(&output, label).1.role(), Role::CheckBox);
        }
        let (_, input) = accesskit_node(&output, "Benis Nachricht");
        assert_eq!(input.role(), Role::TextInput);
        assert_eq!(input.value(), Some(app.speech_draft.as_str()));
        output.drop_without_applying_deltas();
    }
    #[test]
    fn avatar_click_toggles_controls_without_replacing_a_status_or_moving_beni() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.play("daumen", "Build grün");
        app.player.paused = true;
        app.message_source = Some("codex-test".into());
        app.active_remote = true;
        let deadline = app.message_until;
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        assert!(!app.controls_open);
        assert_controls_hidden(&output);
        let avatar = avatar_rect(&output, &app);
        output.drop_without_applying_deltas();

        click(&ctx, &mut app, avatar.center(), 0.1).drop_without_applying_deltas();
        assert!(app.controls_open);
        let output = frame(&ctx, &mut app, 0.2, vec![]);
        for label in ["Hoi!", "Weiter", "Menü", "Beenden"] {
            text_rect(&output, label);
        }
        assert_eq!(avatar_rect(&output, &app), avatar);
        text_rect(&output, "Build grün");
        text_rect(&output, "codex-test");
        output.drop_without_applying_deltas();

        click(&ctx, &mut app, avatar.center(), 0.3).drop_without_applying_deltas();
        assert!(!app.controls_open);
        let output = frame(&ctx, &mut app, 0.4, vec![]);
        assert_controls_hidden(&output);
        assert_eq!(avatar_rect(&output, &app), avatar);
        assert_eq!(app.player.state(), "daumen");
        assert_eq!(app.message, "Build grün");
        assert_eq!(app.message_source.as_deref(), Some("codex-test"));
        assert_eq!(app.message_until, deadline);
        assert!(app.active_remote);
        output.drop_without_applying_deltas();
    }
    #[test]
    fn escape_closes_settings_and_collapses_controls() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.controls_open = true;
        app.menu_open = true;
        frame(&ctx, &mut app, 0.0, vec![]).drop_without_applying_deltas();
        let output = frame(
            &ctx,
            &mut app,
            0.1,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
        assert!(!app.menu_open);
        assert!(!app.controls_open);
        assert_controls_hidden(&output);
        output.drop_without_applying_deltas();
    }
    #[test]
    fn input_map_tracks_visible_avatar_speech_controls_and_settings() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.player.paused = true;
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        let avatar = avatar_rect(&output, &app);
        assert!(app.hit_map.contains(avatar.center()));
        assert!(!app.hit_map.contains(egui::pos2(2.0, 2.0)));
        let close = text_rect(&output, "×").center();
        assert!(app.hit_map.contains(close));
        output.drop_without_applying_deltas();
        app.controls_open = true;
        let output = frame(&ctx, &mut app, 0.1, vec![]);
        let pause = text_rect(&output, "Weiter").center();
        assert!(app.hit_map.contains(pause));
        output.drop_without_applying_deltas();
        app.controls_open = false;
        app.dismiss_speech(Instant::now());
        let output = frame(&ctx, &mut app, 0.2, vec![]);
        assert!(!app.hit_map.contains(pause));
        assert!(!app.hit_map.contains(close));
        output.drop_without_applying_deltas();
        app.menu_open = true;
        frame(&ctx, &mut app, 0.3, vec![]).drop_without_applying_deltas();
        let output = frame(&ctx, &mut app, 0.4, vec![]);
        assert!(
            app.hit_map
                .contains(text_rect(&output, "Hintergrund durchklickbar").center())
        );
        output.drop_without_applying_deltas();
    }
    #[test]
    fn controls_remain_inside_native_window_after_multiple_frames() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.controls_open = true;
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
        for size in BuddySize::ALL {
            let ctx = egui::Context::default();
            ctx.all_styles_mut(|style| style.animation_time = 0.0);
            let mut app = app(&ctx);
            app.size = size;
            app.controls_open = true;
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
            frame(
                &ctx,
                &mut app,
                0.4,
                vec![
                    egui::Event::PointerMoved(egui::pos2(260.0, 300.0)),
                    egui::Event::MouseWheel {
                        phase: egui::TouchPhase::Move,
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -2000.0),
                        modifiers: Default::default(),
                    },
                ],
            )
            .drop_without_applying_deltas();
            let output = frame(&ctx, &mut app, 0.45, vec![]);
            let send = text_rect(&output, "Beni zeigt die Nachricht").center();
            output.drop_without_applying_deltas();
            click(&ctx, &mut app, send, 0.5).drop_without_applying_deltas();
            assert_eq!(app.player.state(), "reden");
            assert_eq!(app.message, "Hoi von den Tests!");
        }
    }
    #[test]
    fn dragging_avatar_requests_a_native_window_drag() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        let pos = avatar_rect(&output, &app).center();
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
        let output = frame(
            &ctx,
            &mut app,
            0.3,
            vec![egui::Event::PointerButton {
                pos: pos + egui::vec2(30.0, 0.0),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        );
        assert!(!app.controls_open, "dragging must not toggle controls");
        assert_controls_hidden(&output);
        output.drop_without_applying_deltas();
    }
    #[test]
    fn pause_resume_and_quit_work_through_real_egui_responses() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.controls_open = true;
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
        assert_controls_hidden(&output);
        let avatar = text_rect(&output, "Beni").center();
        output.drop_without_applying_deltas();
        click(&ctx, &mut app, avatar, 0.5).drop_without_applying_deltas();
        assert!(
            app.controls_open,
            "fallback avatar must still open controls"
        );
        let output = frame(&ctx, &mut app, 0.6, vec![]);
        let menu = text_rect(&output, "Menü").center();
        output.drop_without_applying_deltas();
        click(&ctx, &mut app, menu, 0.7).drop_without_applying_deltas();
        assert!(app.menu_open);
        app.menu_open = false;
        let output = frame(&ctx, &mut app, 0.8, vec![]);
        let quit = text_rect(&output, "Beenden").center();
        output.drop_without_applying_deltas();
        let output = click(&ctx, &mut app, quit, 0.9);
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
            Action::DoNotDisturb,
            Action::Settings,
            Action::History,
            Action::Quit,
        ] {
            app.tray_sender.try_send(action).unwrap();
            let output = ctx.run_logic(&egui::RawInput::default(), |ctx| {
                eframe::App::logic(&mut app, ctx, &mut native_frame);
            });
            match action {
                Action::Show | Action::Recover | Action::Settings | Action::History => {
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
                    } else if action == Action::History {
                        assert!(app.history_player.is_some());
                        assert_eq!(app.player.state(), "nachdenken");
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
                Action::DoNotDisturb => assert!(app.do_not_disturb),
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
        for size in BuddySize::ALL {
            let ctx = egui::Context::default();
            ctx.all_styles_mut(|style| style.animation_time = 0.0);
            let mut app = app(&ctx);
            app.size = size;
            app.controls_open = true;
            let text = "Beni erklärt den Schritt.\n".repeat(15);
            app.play("reden", &text);
            app.message_source = Some("W".repeat(48));
            assert!(app.message_until.duration_since(Instant::now()) > Duration::from_secs(20));
            let output = frame(&ctx, &mut app, 0.0, vec![]);
            for label in ["Hoi!", "Pause", "Menü", "Beenden", "×"] {
                let rect = text_rect(&output, label);
                assert!(
                    rect.min.y >= 0.0 && rect.max.y <= size.dimensions().y,
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
    }

    #[test]
    fn quiet_mode_defaults_off_and_round_trips_without_changing_receiving_or_pause() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        let mut storage = TestStorage::default();
        assert!(!BuddyApp::load_do_not_disturb(None));
        assert!(!BuddyApp::load_do_not_disturb(Some(&storage)));
        storage.0.insert("do-not-disturb".into(), "broken".into());
        assert!(!BuddyApp::load_do_not_disturb(Some(&storage)));
        app.ipc_enabled = true;
        app.player.paused = true;
        for quiet in [true, false] {
            app.set_do_not_disturb(quiet, Instant::now());
            eframe::App::save(&mut app, &mut storage);
            assert_eq!(BuddyApp::load_do_not_disturb(Some(&storage)), quiet);
            assert!(app.ipc_enabled && app.player.paused);
        }
    }

    #[test]
    fn quiet_mode_dismisses_remote_speech_but_preserves_errors_and_manual_actions() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.play("jubeln", "Erfolg");
        app.active_remote = true;
        app.message_source = Some("test".into());
        app.player.paused = true;
        let now = Instant::now();
        app.report_error("Testfehler".into(), now);
        app.set_do_not_disturb(true, now);
        assert!(!app.active_remote);
        assert!(app.message_until <= now);
        assert!(app.message_source.is_none());
        assert_eq!(app.player.state(), "warten");
        assert!(app.player.paused);
        assert_eq!(app.error.as_ref().unwrap().message, "Testfehler");
        app.play("hallo", "Hoi von mir");
        let deadline = app.message_until;
        app.set_do_not_disturb(true, Instant::now());
        assert_eq!(app.message, "Hoi von mir");
        assert_eq!(app.message_until, deadline);
        assert_eq!(app.player.state(), "hallo");
        assert!(!app.hidden);
    }

    #[test]
    fn quiet_mode_has_accessible_controls_and_a_visible_hint_at_every_size() {
        for size in BuddySize::ALL {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            ctx.all_styles_mut(|style| style.animation_time = 0.0);
            let mut app = app(&ctx);
            app.size = size;
            app.menu_open = true;
            frame(&ctx, &mut app, 0.0, vec![]).drop_without_applying_deltas();
            let output = frame(&ctx, &mut app, 0.1, vec![]);
            let checkbox = accesskit_node(&output, "Nicht stören").0;
            output.drop_without_applying_deltas();
            frame(&ctx, &mut app, 0.2, vec![accessible_click(checkbox)])
                .drop_without_applying_deltas();
            assert!(app.do_not_disturb);
            app.menu_open = false;
            app.controls_open = true;
            app.dismiss_speech(Instant::now());
            let output = frame(&ctx, &mut app, 0.3, vec![]);
            for label in ["Nicht stören ist aktiv", "Hoi!", "Pause", "Menü", "Beenden"] {
                let rect = text_rect(&output, label);
                assert!(
                    rect.min.y >= 0.0 && rect.max.y <= size.dimensions().y,
                    "{label}: {rect:?}"
                );
            }
            output.drop_without_applying_deltas();
        }
    }

    #[test]
    fn quiet_mode_holds_all_priorities_and_expires_old_statuses_before_resuming() {
        use buddy_core::notifications::{Notification, Priority};
        let endpoint = common::TestEndpoint::new();
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.server = Some(buddy_ipc::Server::start_at(endpoint.endpoint.clone(), || {}).unwrap());
        app.ipc_enabled = true;
        app.hidden = true;
        let mut native_frame = eframe::Frame::_new_kittest();
        app.tray_sender.try_send(Action::DoNotDisturb).unwrap();
        let _ = ctx.run_logic(&egui::RawInput::default(), |ctx| {
            eframe::App::logic(&mut app, ctx, &mut native_frame);
        });
        assert!(app.do_not_disturb && app.hidden);
        for (text, priority, ttl_s) in [
            ("Normal", Priority::Normal, 30),
            ("Wichtig", Priority::High, 30),
            ("Veraltet", Priority::High, 1),
        ] {
            let mut message = Notification::new(text);
            message.priority = priority;
            message.ttl_s = ttl_s;
            buddy_ipc::notify(&endpoint.endpoint, message).unwrap();
        }
        app.hidden = false;
        app.deliver_status(true, Instant::now());
        assert!(!app.active_remote);
        assert_eq!(app.server.as_ref().unwrap().pending(), 3);
        std::thread::sleep(Duration::from_millis(1100));
        app.tray_sender.try_send(Action::DoNotDisturb).unwrap();
        let _ = ctx.run_logic(&egui::RawInput::default(), |ctx| {
            eframe::App::logic(&mut app, ctx, &mut native_frame);
        });
        assert!(!app.do_not_disturb);
        assert_eq!(app.message, "Wichtig");
        assert_eq!(app.server.as_ref().unwrap().pending(), 1);
        app.dismiss_speech(Instant::now());
        app.deliver_status(true, Instant::now());
        assert_eq!(app.message, "Normal");
        assert_eq!(app.server.as_ref().unwrap().pending(), 0);
    }

    #[test]
    fn reading_history_loops_thinking_and_restores_animation_and_pause_without_replacing_speech() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.play("jubeln", "Build fertig");
        app.player.advance(&app.engine, 100.0, 1.0).unwrap();
        app.message_source = Some("build".into());
        app.active_remote = true;
        let deadline = app.message_until;
        app.open_history();
        assert_eq!(app.player.state(), "nachdenken");
        app.player.advance(&app.engine, 100_000.0, 1.0).unwrap();
        assert_eq!(
            app.player.state(),
            "nachdenken",
            "reading loops the gesture"
        );
        let time = app.player.elapsed_ms();
        app.open_history();
        assert_eq!(
            app.player.elapsed_ms(),
            time,
            "reopening must not restart it"
        );
        app.player.paused = true;
        app.close_history();
        assert_eq!(app.player.state(), "jubeln");
        assert_eq!(app.player.elapsed_ms(), 100.0);
        assert!(app.player.paused && app.active_remote);
        assert_eq!(app.message_until, deadline);
        assert_eq!(app.message_source.as_deref(), Some("build"));
        app.open_history();
        assert!(app.player.paused, "an existing pause stays active");
        app.set_do_not_disturb(true, Instant::now());
        assert_eq!(app.player.state(), "nachdenken");
        assert!(!app.active_remote);
        app.close_history();
        assert_eq!(app.player.state(), "warten");
        assert!(app.player.paused);
        app.open_history();
        app.play("hallo", "Eigene Nachricht");
        assert!(app.history_player.is_none());
        assert_eq!(app.player.state(), "hallo");
        app.open_history();
        frame(
            &ctx,
            &mut app,
            0.0,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
        )
        .drop_without_applying_deltas();
        assert!(app.history_player.is_none());
        assert!(!app.controls_open && !app.menu_open);
        assert_eq!(app.player.state(), "hallo");
    }

    #[test]
    fn history_opens_from_accessible_menu_and_scrolls_long_text_at_all_sizes() {
        for size in BuddySize::ALL {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            ctx.all_styles_mut(|style| style.animation_time = 0.0);
            let mut app = app(&ctx);
            app.size = size;
            app.do_not_disturb = true;
            app.menu_open = true;
            let text = format!("{}Ende", "WWW ".repeat(69));
            let mut message = buddy_core::notifications::Notification::new(&text);
            message.source = "W".repeat(48);
            message.priority = buddy_core::notifications::Priority::High;
            app.history.extend([buddy_ipc::ReceivedNotification {
                message,
                received_at: Instant::now(),
            }]);
            frame(&ctx, &mut app, 0.0, vec![]).drop_without_applying_deltas();
            let output = frame(&ctx, &mut app, 0.1, vec![]);
            let open = accesskit_node(&output, "Verlauf öffnen").0;
            output.drop_without_applying_deltas();
            frame(&ctx, &mut app, 0.2, vec![accessible_click(open)]).drop_without_applying_deltas();
            assert!(app.history_player.is_some());
            assert!(!app.menu_open);
            assert_eq!(app.player.state(), "nachdenken");
            let mut output = frame(&ctx, &mut app, 0.3, vec![]);
            output.textures_delta.clear();
            assert_eq!(
                accesskit_node(&output, "Beni: Bedienleiste zuklappen")
                    .1
                    .description(),
                Some("Animation: Denken")
            );
            for label in [
                "Meldungsverlauf (1)",
                "Leeren",
                "Hoi!",
                "Pause",
                "Menü",
                "Beenden",
            ] {
                let rect = text_rect(&output, label);
                assert!(
                    rect.min.y >= 0.0 && rect.max.y <= size.dimensions().y,
                    "{size:?} {label}: {rect:?}"
                );
                assert!(
                    app.hit_map.contains(rect.center()),
                    "history/controls must remain clickable"
                );
            }
            let avatar = avatar_rect(&output, &app);
            assert!(
                avatar.height() > 0.0
                    && avatar.max.y < text_rect(&output, "Meldungsverlauf (1)").min.y
            );
            let (before, clip) = output
                .shapes
                .iter()
                .find_map(|s| find_text(&s.shape, &text).map(|r| (r, s.clip_rect)))
                .unwrap();
            assert!(
                before.max.y > clip.max.y,
                "{size:?}: long text must require scrolling: {before:?} / {clip:?}"
            );
            output.drop_without_applying_deltas();
            frame(
                &ctx,
                &mut app,
                0.4,
                vec![
                    egui::Event::PointerMoved(clip.center()),
                    egui::Event::MouseWheel {
                        phase: egui::TouchPhase::Move,
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -2000.0),
                        modifiers: Default::default(),
                    },
                ],
            )
            .drop_without_applying_deltas();
            let output = frame(&ctx, &mut app, 0.5, vec![]);
            let (after, clip) = output
                .shapes
                .iter()
                .find_map(|s| find_text(&s.shape, &text).map(|r| (r, s.clip_rect)))
                .unwrap();
            assert!(
                after.max.y <= clip.max.y + 1.0,
                "last line must be reachable"
            );
            let close = accesskit_node(&output, "Verlauf schliessen").0;
            output.drop_without_applying_deltas();
            frame(&ctx, &mut app, 0.6, vec![accessible_click(close)])
                .drop_without_applying_deltas();
            assert!(app.history_player.is_none());
            assert_eq!(app.player.state(), "warten");
        }
    }

    #[test]
    fn history_keeps_hidden_quiet_expired_receipts_and_clears_without_consuming_delivery() {
        use buddy_core::notifications::Notification;
        let endpoint = common::TestEndpoint::new();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = app(&ctx);
        app.server = Some(buddy_ipc::Server::start_at(endpoint.endpoint.clone(), || {}).unwrap());
        app.ipc_enabled = true;
        app.hidden = true;
        app.do_not_disturb = true;
        let mut expired = Notification::new("Kurzfristig");
        expired.ttl_s = 1;
        buddy_ipc::notify(&endpoint.endpoint, expired).unwrap();
        buddy_ipc::notify(&endpoint.endpoint, Notification::new("Bleibt im Verlauf")).unwrap();
        std::thread::sleep(Duration::from_millis(1100));
        let mut native_frame = eframe::Frame::_new_kittest();
        let logic = |app: &mut BuddyApp, frame: &mut eframe::Frame| {
            let _ = ctx.run_logic(&egui::RawInput::default(), |ctx| {
                eframe::App::logic(app, ctx, frame)
            });
        };
        logic(&mut app, &mut native_frame);
        assert_eq!(app.history.len(), 2);
        assert_eq!(app.server.as_ref().unwrap().pending(), 1);
        assert!(!app.active_remote);
        app.tray_sender.try_send(Action::History).unwrap();
        logic(&mut app, &mut native_frame);
        assert!(!app.hidden);
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        assert!(
            text_rect(&output, "Bleibt im Verlauf").min.y < text_rect(&output, "Kurzfristig").min.y,
            "newest receipt is shown first"
        );
        output.drop_without_applying_deltas();
        buddy_ipc::notify(&endpoint.endpoint, Notification::new("Neu beim Lesen")).unwrap();
        logic(&mut app, &mut native_frame);
        logic(&mut app, &mut native_frame);
        assert_eq!(app.history.len(), 3, "receipts are collected once");
        assert_eq!(app.player.state(), "nachdenken");
        assert_eq!(app.server.as_ref().unwrap().pending(), 2);
        let mut storage = TestStorage::default();
        eframe::App::save(&mut app, &mut storage);
        assert!(
            storage
                .0
                .values()
                .all(|v| !v.contains("Bleibt im Verlauf") && !v.contains("Neu beim Lesen"))
        );
        let output = frame(&ctx, &mut app, 0.1, vec![]);
        let clear = accesskit_node(&output, "Leeren").0;
        output.drop_without_applying_deltas();
        buddy_ipc::notify(&endpoint.endpoint, Notification::new("Noch gepuffert")).unwrap();
        frame(&ctx, &mut app, 0.2, vec![accessible_click(clear)]).drop_without_applying_deltas();
        logic(&mut app, &mut native_frame);
        assert_eq!(
            app.history.len(),
            0,
            "clear includes receipts still in the server buffer"
        );
        assert_eq!(app.server.as_ref().unwrap().pending(), 3);
        let output = frame(&ctx, &mut app, 0.3, vec![]);
        text_rect(&output, "Hier erscheinen deine Statusmeldungen.");
        output.drop_without_applying_deltas();
        buddy_ipc::notify(&endpoint.endpoint, Notification::new("Beim Abschalten")).unwrap();
        app.player.paused = true;
        app.set_do_not_disturb(false, Instant::now());
        app.close_history();
        logic(&mut app, &mut native_frame);
        assert!(
            app.player.paused && app.active_remote,
            "resuming status delivery must not lift the reading pause"
        );
        app.ipc_enabled = false;
        logic(&mut app, &mut native_frame);
        assert!(app.server.is_none());
        assert_eq!(
            app.history.len(),
            1,
            "turning off reception keeps the last receipt"
        );
    }

    #[test]
    fn ipc_wakes_a_settled_paused_ui_immediately_and_preserves_pause() {
        let endpoint = common::TestEndpoint::new();
        let ctx = egui::Context::default();
        ctx.all_styles_mut(|style| style.animation_time = 0.0);
        let mut app = app(&ctx);
        app.player.paused = true;
        app.message_until = Instant::now();
        for index in 0..6 {
            frame(&ctx, &mut app, index as f64 * 0.02, vec![]).drop_without_applying_deltas();
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        ctx.set_request_repaint_callback(move |info| {
            let _ = sender.send(info.delay);
        });
        let wake = ctx.clone();
        app.server = Some(
            buddy_ipc::Server::start_at(endpoint.endpoint.clone(), move || wake.request_repaint())
                .unwrap(),
        );
        app.ipc_enabled = true;
        buddy_ipc::notify(
            &endpoint.endpoint,
            buddy_core::notifications::Notification::new("Build grün nach Pause"),
        )
        .unwrap();
        assert_eq!(
            receiver.recv_timeout(Duration::from_secs(1)).unwrap(),
            Duration::ZERO
        );
        let mut native_frame = eframe::Frame::_new_kittest();
        let _ = ctx.run_logic(&egui::RawInput::default(), |ctx| {
            eframe::App::logic(&mut app, ctx, &mut native_frame);
        });
        assert!(app.active_remote && app.player.paused);
        let output = frame(&ctx, &mut app, 0.2, vec![]);
        text_rect(&output, "Build grün nach Pause");
        output.drop_without_applying_deltas();
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
