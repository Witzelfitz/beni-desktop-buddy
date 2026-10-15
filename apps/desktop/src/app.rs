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
    idle: crate::idle::Idle,
    texture: TextureHandle,
    atlas_alpha: Vec<u8>,
    hit_map: crate::hit_map::HitMap,
    input: crate::input::InputGuard,
    click_through: bool,
    size: BuddySize,
    size_pending: bool,
    hotkeys: crate::hotkeys::Hotkeys,
    history: crate::history::History,
    timer: crate::timer::Timer,
    reminders: crate::reminders::Reminders,
    updates: crate::updater::Updater,
    updates_info: bool,
    history_player: Option<Player>,
    chat: crate::chat::Chat,
    chat_player: Option<Player>,
    chat_cue: Option<crate::chat::Cue>,
    chat_layout_open: bool,
    rest_position: Option<[f32; 2]>,
    initial_geometry: bool,
    space_above: Option<f32>,
    work_area: Option<Vec2>,
    last_monitor_size: Option<Vec2>,
    settings: Settings,
    motion: crate::motion::MotionPreference,
    system_motion: crate::motion::SystemMotion,
    last_tick: Instant,
    message: String,
    message_until: Instant,
    message_source: Option<String>,
    active_status: bool,
    do_not_disturb: bool,
    ipc_enabled: bool,
    server: Option<buddy_ipc::Server>,
    hidden: bool,
    controls_open: bool,
    menu_open: bool,
    settings_focused: bool,
    settings_page: crate::settings_window::Page,
    card_focus: bool,
    card_position: Option<egui::Pos2>,
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
            idle: crate::idle::Idle::default(),
            texture,
            atlas_alpha: image.pixels.iter().map(|pixel| pixel.a()).collect(),
            hit_map: crate::hit_map::HitMap::default(),
            input: crate::input::InputGuard::default(),
            click_through: cc.storage.and_then(|s| eframe::get_value(s, "click-through")).unwrap_or(true),
            settings,
            motion: crate::motion::MotionPreference::load(cc.storage),
            system_motion: crate::motion::SystemMotion::start(),
            size: BuddySize::load(cc.storage),
            size_pending: true,
            hotkeys: crate::hotkeys::Hotkeys::load(cc.storage),
            history: crate::history::History::default(),
            timer: crate::timer::Timer::load(cc.storage),
            reminders: crate::reminders::Reminders::default(),
            updates: crate::updater::Updater::default(),
            updates_info: false,
            history_player: None,
            chat: crate::chat::Chat::load(cc.storage),
            chat_player: None,
            chat_cue: None,
            chat_layout_open: false,
            rest_position: cc.storage.and_then(|s| eframe::get_value::<[f32; 2]>(s, "buddy-rest-position"))
                .filter(|p| p.iter().all(|n| n.is_finite())),
            initial_geometry: true,
            space_above: None,
            work_area: None,
            last_monitor_size: None,
            last_tick: Instant::now(),
            message: "Hallo! Ich bin Beni. Klick mich für die Bedienung an oder zieh mich an deinen Lieblingsplatz.".into(),
            message_until: Instant::now() + Duration::from_secs(10),
            message_source: None,
            active_status: false,
            do_not_disturb: Self::load_do_not_disturb(cc.storage),
            ipc_enabled: cc
                .storage
                .and_then(|s| eframe::get_value(s, "status-receiving"))
                .unwrap_or(false),
            server: None,
            hidden: false,
            controls_open: false,
            menu_open: false,
            settings_focused: false,
            settings_page: crate::settings_window::Page::default(),
            card_focus: false,
            card_position: None,
            speech_draft: "Schön, dass du da bist!".into(),
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
        self.stop_chat_animation();
        self.close_history();
        if let Err(e) = self.player.play(&self.engine, state, true) {
            self.report_error(e.to_string(), Instant::now());
            return;
        }
        self.idle.reset();
        self.error = None;
        self.message = message.chars().take(400).collect();
        self.message_until = Instant::now() + Duration::from_millis(reading_time_ms(&self.message));
        self.message_source = None;
        self.active_status = false;
    }
    fn report_error(&mut self, message: String, now: Instant) {
        self.error = Some(TransientError {
            message,
            until: now + Duration::from_secs(6),
        });
    }
    fn save_for_update(&mut self, storage: &mut dyn eframe::Storage) -> std::io::Result<()> {
        eframe::App::save(self, storage);
        storage.flush_sync()?;
        self.chat.cancel();
        self.stop_receiving();
        Ok(())
    }
    fn expire_error(&mut self, now: Instant) {
        if self.error.as_ref().is_some_and(|error| now >= error.until) {
            self.error = None;
        }
    }
    fn dismiss_speech(&mut self, now: Instant) {
        self.error = None;
        self.message_until = now;
        self.active_status = false;
    }
    fn repaint_delay(&self, now: Instant, visible: bool) -> Duration {
        // Keep the once-per-second topology check, history ages and reduced-motion
        // one-shots progressing. Input, IPC and tray actions wake the UI directly.
        let housekeeping = Duration::from_secs(1);
        if !visible {
            return housekeeping;
        }
        let mut delay = if self.motion.reduced(self.settings.reduced_motion) || self.player.paused {
            housekeeping
        } else if self.player.state() == "warten"
            || (self.history_player.is_none() && self.idle.owns(&self.player))
        {
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
    fn bubble_state(&self, visible: bool, now: Instant) -> crate::bubble_policy::State {
        crate::bubble_policy::State {
            visible,
            quiet: self.do_not_disturb,
            history: self.history_player.is_some(),
            conversation: self.chat.open,
            error: self.error.is_some(),
            speech_active: now < self.message_until,
            status_active: self.active_status,
        }
    }
    fn deliver_status(&mut self, visible: bool, now: Instant) {
        let policy = self.bubble_state(visible, now);
        if !policy.accepts_ipc() {
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
            self.active_status = true;
            return;
        }
        // Local notices wait for all visible speech, including manual text.
        if policy.accepts_local() {
            if let Some(phase) = self.timer.take_notice(now) {
                self.show_local_notice(phase.message(), phase.animation(), "timer", now);
            } else if let Some(reminder) = self.reminders.take_notice(now) {
                self.show_local_notice(&reminder.text, "idee", "erinnerung", now);
            }
        }
    }
    fn show_local_notice(&mut self, text: &str, state: &str, source: &str, now: Instant) {
        let paused = self.player.paused;
        self.play(state, text);
        self.player.paused = paused;
        self.message_source = Some(source.into());
        self.message_until = now + Duration::from_millis(reading_time_ms(text));
        self.active_status = true;
    }
    fn record_local_notice(&mut self, text: &str, state: &str, source: &str, now: Instant) {
        let mut message = buddy_core::notifications::Notification::new(text);
        message.state = Some(state.into());
        message.source = source.into();
        self.history.extend([buddy_ipc::ReceivedNotification {
            message,
            received_at: now,
        }]);
    }
    fn advance_timer(&mut self, now: Instant) {
        if let Some((phase, _deadline)) = self.timer.tick(now) {
            self.record_local_notice(phase.message(), phase.animation(), "timer", now);
        }
    }
    fn advance_reminders(&mut self, now: Instant) {
        for reminder in self.reminders.tick(now) {
            self.record_local_notice(&reminder.text, "idee", "erinnerung", now);
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
        self.chat.close();
        self.stop_chat_animation();
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
        if enabled && self.active_status {
            self.message_until = now;
            self.message_source = None;
            self.active_status = false;
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
        let mut effective = self.settings.clone();
        effective.reduced_motion = self.motion.reduced(self.settings.reduced_motion);
        let pose = self.player.pose(&self.engine, &effective)?;
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
    fn paint_avatar(&mut self, ui: &mut egui::Ui, stage: Rect, wheel_area: Rect) -> Rect {
        // Register interactions first: a failed draw must never remove the drag/click target.
        let response = ui.interact(stage, ui.id().with("avatar"), Sense::click_and_drag());
        if response.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if response.clicked() {
            if self.settings_page.is_card() {
                self.menu_open = false;
            }
            self.controls_open = !self.controls_open;
            if !self.controls_open {
                self.close_history();
            }
        }
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Button,
                response.enabled(),
                if self.history_player.is_some() {
                    "Beni: Verlauf schliessen"
                } else if self.controls_open {
                    "Beni: Menü-Rad schliessen"
                } else {
                    "Beni: Menü-Rad öffnen"
                },
            )
        });
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_expanded(self.controls_open);
            if let Some(animation) = self.engine.animations.get(self.player.state()) {
                node.set_description(format!("Animation: {}", animation.label));
            }
        });
        response.on_hover_text(if self.history_player.is_some() {
            "Ziehen: verschieben · Klicken: Verlauf schliessen"
        } else if self.controls_open {
            "Ziehen: verschieben · Klicken: Menü-Rad schliessen"
        } else {
            "Ziehen: verschieben · Klicken: Menü-Rad öffnen"
        });
        let mut mesh = self.avatar_mesh(stage);
        let original = mesh
            .as_ref()
            .map(|mesh| Rect::from_points(&mesh.vertices.iter().map(|v| v.pos).collect::<Vec<_>>()))
            .unwrap_or(stage.shrink(35.0));
        let mut avatar = original;
        if self.controls_open
            && self.history_player.is_none()
            && let Some(layout) = crate::action_wheel::Layout::new(wheel_area, original)
            && let Ok(mesh) = &mut mesh
        {
            let scale = layout.avatar.width() / original.width();
            for vertex in &mut mesh.vertices {
                vertex.pos = layout.avatar.min + (vertex.pos - original.min) * scale;
            }
            avatar = layout.avatar;
        }
        match mesh {
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
        avatar
    }
    fn controls(&mut self, ui: &mut egui::Ui, area: Rect, avatar: Rect) {
        use crate::action_wheel::{Action as Wheel, Layout};
        let mut selected = None;
        let layout = Layout::new(area, avatar);
        let radial = layout.is_some();
        if let Some(layout) = layout {
            for (action, rect) in Wheel::ALL.into_iter().zip(layout.buttons) {
                if crate::action_wheel::button(ui, rect, action, self.do_not_disturb).clicked() {
                    selected = Some(action);
                }
                self.hit_map.add_rect(rect, 14.0);
            }
        } else {
            // A compositor may constrain the viewport below our requested size.
            // Keep labelled actions reachable in a bounded scrolling fallback.
            ui.scope_builder(egui::UiBuilder::new().max_rect(area), |ui| {
                egui::Frame::new()
                    .fill(Color32::from_rgb(24, 40, 44))
                    .corner_radius(12)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height((area.height() - 8.0).max(1.0))
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                                let columns = if ui.available_width() >= 176.0 { 2 } else { 1 };
                                let width = (ui.available_width() - 4.0 * (columns - 1) as f32)
                                    / columns as f32;
                                for row in Wheel::ALL.chunks(columns) {
                                    ui.horizontal(|ui| {
                                        for &action in row {
                                            let (rect, _) = ui.allocate_exact_size(
                                                egui::vec2(width, 44.0),
                                                Sense::hover(),
                                            );
                                            if crate::action_wheel::button(
                                                ui,
                                                rect,
                                                action,
                                                self.do_not_disturb,
                                            )
                                            .clicked()
                                            {
                                                selected = Some(action);
                                            }
                                        }
                                    });
                                }
                            });
                    });
            });
            self.hit_map.add_rect(area, 12.0);
        }
        let state = if self.timer.has_session() {
            self.timer.status(Instant::now())
        } else if self.reminders.has_tasks() {
            format!(
                "{} Erinnerung{}",
                self.reminders.count(),
                if self.reminders.count() == 1 {
                    ""
                } else {
                    "en"
                }
            )
        } else {
            "Bereit".into()
        };
        let state_rect = Rect::from_center_size(
            egui::pos2(area.center().x, area.bottom() - 19.0),
            egui::vec2((area.width() - 184.0).clamp(88.0, 164.0), 28.0),
        );
        if radial {
            ui.painter()
                .rect_filled(state_rect, 12, crate::action_wheel::FILL);
            ui.put(
                state_rect,
                egui::Label::new(
                    egui::RichText::new(state)
                        .size(11.5)
                        .color(Color32::from_rgb(239, 247, 242)),
                )
                .wrap(),
            );
            self.hit_map.add_rect(state_rect, 6.0);
        }
        if let Some(action) = selected {
            self.controls_open = false;
            match action {
                Wheel::Focus => self.open_page(ui.ctx(), crate::settings_window::Page::Focus),
                Wheel::Reminder => {
                    self.open_page(ui.ctx(), crate::settings_window::Page::Reminders)
                }
                Wheel::History => self.open_history(),
                Wheel::Quiet => self.set_do_not_disturb(!self.do_not_disturb, Instant::now()),
                Wheel::Chat => self.open_chat(ui.ctx()),
                Wheel::Settings => self.open_settings(ui.ctx()),
            }
        }
    }
    fn open_chat(&mut self, ctx: &egui::Context) {
        self.close_history();
        self.menu_open = false;
        self.controls_open = false;
        if self.active_status {
            let paused = self.player.paused;
            if self.player.play(&self.engine, "warten", false).is_ok() {
                self.player.paused = paused;
            }
        }
        self.message_until = Instant::now();
        self.active_status = false;
        self.chat.open(ctx);
    }
    fn stop_chat_animation(&mut self) {
        self.chat_cue = None;
        if let Some(mut previous) = self.chat_player.take() {
            previous.paused = self.player.paused;
            self.player = previous;
        }
    }
    fn sync_chat_animation(&mut self, now: Instant) {
        let state = if self.bubble_state(!self.hidden, now).animates_conversation() {
            self.chat.cue()
        } else {
            None
        };
        let Some(cue) = state else {
            self.stop_chat_animation();
            return;
        };
        if self.chat_player.is_none() {
            self.chat_player = Some(self.player.clone());
            self.idle.reset();
        }
        if self.chat_cue != Some(cue) {
            let paused = self.player.paused;
            if let Err(error) = self
                .player
                .play(&self.engine, cue.animation(), cue.one_shot())
            {
                self.stop_chat_animation();
                self.report_error(error.to_string(), now);
            } else {
                self.player.paused = paused;
                self.chat_cue = Some(cue);
            }
        }
    }
    fn open_settings(&mut self, ctx: &egui::Context) {
        self.open_page(ctx, crate::settings_window::Page::Settings);
    }
    fn open_page(&mut self, ctx: &egui::Context, page: crate::settings_window::Page) {
        self.settings_page = page;
        self.card_focus = page.is_card();
        self.card_position = None;
        self.updates_info = false;
        self.close_history();
        if !self.menu_open {
            self.settings_focused = false;
        }
        self.menu_open = true;
        ctx.send_viewport_cmd_to(
            crate::settings_window::id(),
            egui::ViewportCommand::Minimized(false),
        );
        ctx.send_viewport_cmd_to(crate::settings_window::id(), egui::ViewportCommand::Focus);
        ctx.request_repaint();
    }
    fn menu(&mut self, ctx: &egui::Context) {
        if !self.menu_open {
            return;
        }
        if self.settings_page.is_card() {
            self.quick_card(ctx);
            return;
        }
        if ctx.embed_viewports() {
            // Embedded fallback for backends without native viewports, including
            // headless tests. The desktop backend uses the independent window below.
            let mut open = true;
            let available = ctx.content_rect().size() - Vec2::splat(24.0);
            let response = egui::Window::new(self.settings_page.title())
                .open(&mut open)
                .fixed_pos([12.0, 12.0])
                .fixed_size([available.x, available.y.min(360.0)])
                .vscroll(true)
                .collapsible(false)
                .show(ctx, |ui| {
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.menu_open = false;
                    }
                    self.settings_contents(ui);
                });
            if let Some(response) = response {
                self.hit_map.add_rect(response.response.rect, 0.0);
            }
            self.menu_open &= open && self.history_player.is_none();
            return;
        }
        ctx.show_viewport_immediate(
            crate::settings_window::id(),
            crate::settings_window::builder().with_title(self.settings_page.title()),
            |ui, _| {
                let child = ui.ctx().clone();
                let close = child.input(|i| i.viewport().close_requested());
                let escape = child.input(|i| i.key_pressed(egui::Key::Escape));
                if close || (escape && !self.settings_focused) {
                    self.menu_open = false;
                    child.request_repaint_of(egui::ViewportId::ROOT);
                    return;
                }
                if escape && let Some(focused) = child.memory(|m| m.focused()) {
                    child.memory_mut(|m| m.surrender_focus(focused));
                }
                if child.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::Q)) {
                    child
                        .send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
                }
                ui.painter()
                    .rect_filled(ui.max_rect(), 0.0, ui.visuals().window_fill());
                egui::Frame::new().inner_margin(16).show(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    let height = (ui.available_height() - 48.0).max(100.0);
                    egui::ScrollArea::vertical()
                        .id_salt("settings-content")
                        .max_height(height)
                        .auto_shrink([false, false])
                        .show(ui, |ui| self.settings_contents(ui));
                    ui.separator();
                    if ui.button("Schliessen").clicked() {
                        self.menu_open = false;
                        child.request_repaint_of(egui::ViewportId::ROOT);
                    }
                });
                // egui releases focus while processing Escape, before this UI callback.
                self.settings_focused = child.memory(|m| m.focused()).is_some();
            },
        );
    }
    fn quick_card(&mut self, ctx: &egui::Context) {
        if ctx.embed_viewports() {
            let available = ctx.content_rect().size() - Vec2::splat(24.0);
            let size = self.settings_page.card_size().min(available);
            let response = egui::Window::new(self.settings_page.title())
                .title_bar(false)
                .fixed_pos([12.0, 12.0])
                .fixed_size(size)
                .frame(crate::settings_window::card_frame())
                .show(ctx, |ui| {
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.menu_open = false;
                    } else {
                        self.quick_card_contents(ui, false);
                    }
                });
            if let Some(response) = response {
                self.hit_map.add_rect(response.response.rect, 20.0);
            }
            return;
        }
        ctx.show_viewport_immediate(
            crate::settings_window::id(),
            crate::settings_window::card_builder(self.settings_page, self.card_position),
            |ui, _| {
                let child = ui.ctx().clone();
                if child
                    .input(|i| i.viewport().close_requested() || i.key_pressed(egui::Key::Escape))
                {
                    self.menu_open = false;
                    child.request_repaint_of(egui::ViewportId::ROOT);
                    return;
                }
                if child.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::Q)) {
                    child
                        .send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
                }
                crate::settings_window::card_frame()
                    .outer_margin(10)
                    .show(ui, |ui| {
                        ui.set_min_size(ui.available_size());
                        self.quick_card_contents(ui, true);
                    });
                if !self.menu_open {
                    child.request_repaint_of(egui::ViewportId::ROOT);
                }
            },
        );
    }
    fn quick_card_contents(&mut self, ui: &mut egui::Ui, native: bool) {
        crate::settings_window::style(ui);
        ui.horizontal(|ui| {
            let heading = ui.label(
                egui::RichText::new(
                    if self.settings_page == crate::settings_window::Page::Focus {
                        "Zeit für dich"
                    } else {
                        "Deine Erinnerung"
                    },
                )
                .size(19.0)
                .strong(),
            );
            if native && heading.interact(Sense::drag()).drag_started() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let close = ui.add(egui::Button::new("×").min_size(egui::vec2(32.0, 32.0)));
                close.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Button,
                        true,
                        "Eingabekarte schliessen",
                    )
                });
                if close.clicked() {
                    self.menu_open = false;
                }
                close.on_hover_text("Schliessen · Escape");
            });
        });
        ui.add_space(4.0);
        let focus = std::mem::take(&mut self.card_focus);
        let submitted = egui::ScrollArea::vertical()
            .id_salt("quick-card-content")
            .max_height((ui.available_height() - 4.0).max(80.0))
            .auto_shrink([false, true])
            .show(ui, |ui| match self.settings_page {
                crate::settings_window::Page::Focus => self.timer.card(ui, Instant::now(), focus),
                crate::settings_window::Page::Reminders => {
                    self.reminders.card(ui, Instant::now(), focus)
                }
                crate::settings_window::Page::Settings => false,
            })
            .inner;
        if submitted {
            self.menu_open = false;
            self.controls_open = true;
        }
    }
    fn settings_contents(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        if ui.button("Verlauf öffnen").clicked() {
            self.open_history();
        }
        if self.updates_info {
            self.updates.ui(ui);
        }
        self.timer.ui(ui, Instant::now());
        self.reminders.ui(ui, Instant::now());
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
        ui.checkbox(&mut self.motion.follow_system, "Systemeinstellung beachten");
        ui.small(self.motion.status());
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
            ctx.send_viewport_cmd_to(
                egui::ViewportId::ROOT,
                egui::ViewportCommand::WindowLevel(if self.settings.always_on_top {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                }),
            );
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
        ui.small("Alt/Option + Q beendet Beni.");
        if !self.updates_info {
            self.updates.ui(ui);
        }
    }
}
impl BuddyApp {
    fn sync_conversation_geometry(
        &mut self,
        ctx: &egui::Context,
        window: Option<&winit::window::Window>,
    ) {
        let base = self.size.dimensions();
        let actual = ctx.content_rect().size();
        let outer = ctx.input(|i| i.viewport().outer_rect.map(|r| r.min));
        if !self.initial_geometry
            && let Some(origin) = outer
        {
            let position = origin + crate::chat::offset(base, actual);
            self.rest_position = Some([position.x, position.y]);
        }
        let raw_monitor = ctx
            .input(|i| i.viewport().monitor_size)
            .filter(|m| m.x.is_finite() && m.y.is_finite() && m.x > 0.0 && m.y > 0.0);
        let monitor_changed = self.last_monitor_size != raw_monitor;
        self.last_monitor_size = raw_monitor;
        if self.size_pending || self.chat_layout_open != self.chat.open || monitor_changed {
            if let Some(window) = window {
                let geometry = crate::window::conversation_screen(window);
                self.space_above = geometry.map(|g| g.0);
                self.work_area = geometry.map(|g| g.1);
            }
            let monitor = self.work_area.or(raw_monitor);
            let desired =
                crate::chat::dimensions_with_space(base, self.chat.open, monitor, self.space_above);
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(desired));
            let position = if self.initial_geometry {
                self.rest_position.map(|p| egui::pos2(p[0], p[1]))
            } else {
                outer.map(|p| {
                    p + crate::chat::offset(base, actual) - crate::chat::offset(base, desired)
                })
            };
            if let Some(position) = position {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(position));
            }
            self.placement.invalidate();
            self.chat_layout_open = self.chat.open;
            self.size_pending = false;
            self.initial_geometry = false;
        }
    }
    #[cfg(test)]
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        self.show_ui_with_window(ui, None);
    }
    fn show_ui_with_window(&mut self, ui: &mut egui::Ui, window: Option<&winit::window::Window>) {
        let ctx = ui.ctx().clone();
        self.hit_map.reset(ctx.content_rect().size());
        let now = Instant::now();
        let delta = now.duration_since(self.last_tick).as_secs_f64() * 1000.0;
        let occluded = ctx.input(|i| i.viewport().occluded.unwrap_or(false));
        self.last_tick = now;
        self.expire_error(now);
        if let Err(e) = self
            .player
            .advance(&self.engine, delta.min(1000.0), self.settings.speed)
        {
            self.report_error(e.to_string(), Instant::now());
            self.settings = self.settings.clone().sanitize();
        }
        // Reading suspends the idle scheduler along with its saved Player.
        if self.history_player.is_none() && self.chat_player.is_none() {
            let enabled = !self.motion.reduced(self.settings.reduced_motion)
                && !self.menu_open
                && !self.controls_open
                && !self.hidden
                && !occluded
                && now >= self.message_until
                && self.error.is_none();
            if let Err(error) = self.idle.advance(
                &self.engine,
                &mut self.player,
                delta.min(1000.0),
                self.settings.speed,
                enabled,
            ) {
                self.report_error(error.to_string(), now);
            }
        }
        let quit = ctx.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::Q));
        if quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.chat.escape(&ctx);
            self.controls_open = false;
            self.close_history();
        }
        let mut base = self.size.dimensions();
        let bounds = ctx.content_rect();
        let side = self.chat.open && crate::chat::beside(base, bounds.size());
        if self.chat.open && !side && bounds.height() != base.y && bounds.height() < base.y + 150.0
        {
            base.y = (bounds.height() - 200.0).max(160.0);
        }
        let extra = (bounds.height() - base.y).max(0.0);
        if self.chat.open && (extra >= 150.0 || side) {
            let bubble = if side {
                Rect::from_min_max(
                    bounds.min + egui::vec2(8.0, 8.0),
                    egui::pos2(bounds.max.x - base.x - 20.0, bounds.max.y - 20.0),
                )
            } else {
                Rect::from_min_max(
                    egui::pos2(bounds.min.x + 8.0, bounds.min.y + 8.0),
                    egui::pos2(
                        bounds.max.x - 8.0,
                        bounds.min.y + extra + if self.error.is_none() { 58.0 } else { -20.0 },
                    ),
                )
            };
            ui.scope_builder(egui::UiBuilder::new().max_rect(bubble), |ui| {
                let rect = self.chat.show(ui, side);
                self.hit_map.add_rect(rect, 16.0);
            });
        }
        let buddy =
            Rect::from_min_size(bounds.min + crate::chat::offset(base, bounds.size()), base);
        ui.scope_builder(egui::UiBuilder::new().max_rect(buddy), |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                if self.history_player.is_none() {
                    self.speech(ui, now);
                }
                if self.do_not_disturb {
                    let hint = ui.small("Nicht stören ist aktiv");
                    self.hit_map.add_rect(hint.rect, 0.0);
                }
                let history_height = if self.history_player.is_some() {
                    180.0
                } else {
                    0.0
                };
                let stage_height = (ui.available_height() - 52.0 - history_height).max(100.0);
                let (_, stage) = ui.allocate_space(Vec2::new(ui.available_width(), stage_height));
                let wheel_area = Rect::from_min_max(
                    stage.min + egui::vec2(8.0, 0.0),
                    egui::pos2(stage.max.x - 8.0, buddy.max.y - 8.0),
                )
                .intersect(bounds.shrink(8.0));
                let avatar = self.paint_avatar(ui, stage, wheel_area);
                if self.history_player.is_some() {
                    self.history_ui(ui, now);
                }
                if self.controls_open && self.history_player.is_none() {
                    self.controls(ui, wheel_area, avatar);
                }
            });
        });
        if self.menu_open && self.settings_page.is_card() && self.card_position.is_none() {
            self.card_position = window.and_then(|window| {
                crate::window::quick_card_position(
                    window,
                    self.size.dimensions(),
                    self.settings_page.card_size(),
                )
            });
        }
        self.menu(&ctx);
        self.sync_conversation_geometry(&ctx, window);
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
            Action::Show | Action::Recover | Action::Settings | Action::History | Action::Chat => {
                self.hidden = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                if let Some(window) = window {
                    crate::window::restore(window, &mut self.placement, action == Action::Recover);
                }
                if action == Action::Settings {
                    self.open_settings(ctx);
                } else if action == Action::History {
                    self.open_history();
                } else if action == Action::Chat {
                    self.open_chat(ctx);
                }
            }
            Action::Hide => {
                if self.settings_page.is_card() {
                    self.menu_open = false;
                }
                self.hidden = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
            Action::Pause => self.player.paused = !self.player.paused,
            Action::DoNotDisturb => {
                self.set_do_not_disturb(!self.do_not_disturb, Instant::now());
            }
            Action::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Action::CheckUpdates => {
                if self.updates.check().is_err() {
                    // Explain pending feeds/unsupported developer builds in the
                    // independent settings window, including when Beni is hidden.
                    self.handle_tray(Action::Settings, ctx, window);
                    self.updates_info = true;
                }
            }
        }
    }
}
impl eframe::App for BuddyApp {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(value) = self.system_motion.poll(ctx, Instant::now()) {
            self.motion.system = value;
        }
        let window = frame.winit_window().cloned();
        if let Some(window) = window.as_deref() {
            self.updates.initialize();
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
        self.advance_timer(Instant::now());
        self.advance_reminders(Instant::now());
        self.chat.poll();
        self.sync_chat_animation(Instant::now());
        self.updates.poll();
        let busy = self.timer.has_session() || self.reminders.has_tasks() || self.chat.busy();
        if self.updates.needs_confirmation(busy) && !self.updates_info {
            self.handle_tray(Action::Settings, ctx, window.as_deref());
            self.updates_info = true;
        }
        if self.updates.ready_to_install(busy) {
            // Persist preferences before Sparkle terminates the process. Do not
            // change the stored IPC opt-in while closing its live transport.
            if let Some(storage) = frame.storage_mut()
                && self.save_for_update(storage).is_ok()
            {
                self.updates.finish_install();
                return;
            }
            self.updates.defer_install();
            self.report_error("Einstellungen konnten nicht gespeichert werden. Das Update wartet; bitte prüfe den Speicherort und versuche es erneut.".into(), Instant::now());
            self.handle_tray(Action::Settings, ctx, window.as_deref());
            self.updates_info = true;
        }
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
            tray.set_updates_enabled(self.updates.menu_enabled());
        }
        self.expire_error(Instant::now());
        // Also check topology when hidden/occluded; avoid an animation loop in that state.
        ctx.request_repaint_after(Duration::from_secs(1));
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let window = frame.winit_window().cloned();
        self.show_ui_with_window(ui, window.as_deref());
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
        self.motion.save(storage);
        eframe::set_value(storage, "status-receiving", &self.ipc_enabled);
        eframe::set_value(storage, "click-through", &self.click_through);
        eframe::set_value(storage, "do-not-disturb", &self.do_not_disturb);
        self.size.save(storage);
        self.hotkeys.save(storage);
        self.timer.save(storage);
        self.chat.save(storage);
        if let Some(position) = self.rest_position {
            eframe::set_value(storage, "buddy-rest-position", &position);
        }
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
            idle: crate::idle::Idle::default(),
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
            timer: crate::timer::Timer::default(),
            reminders: crate::reminders::Reminders::default(),
            updates: crate::updater::Updater::default(),
            updates_info: false,
            history_player: None,
            chat: crate::chat::Chat::load(None),
            chat_player: None,
            chat_cue: None,
            chat_layout_open: false,
            rest_position: None,
            initial_geometry: true,
            space_above: None,
            work_area: None,
            last_monitor_size: None,
            settings: Settings::default(),
            motion: crate::motion::MotionPreference::load(None),
            system_motion: crate::motion::SystemMotion::default(),
            last_tick: Instant::now(),
            message: "Hallo aus der Test-Sprechblase!".into(),
            message_until: Instant::now() + Duration::from_secs(10),
            message_source: None,
            active_status: false,
            do_not_disturb: false,
            ipc_enabled: false,
            server: None,
            hidden: false,
            controls_open: false,
            menu_open: false,
            settings_focused: false,
            settings_page: crate::settings_window::Page::default(),
            card_focus: false,
            card_position: None,
            speech_draft: "Hallo von den Tests!".into(),
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
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, app.size.dimensions())),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ui| app.show_ui(ui),
        );
        output.textures_delta.clear();
        output
    }
    struct NativeSettings {
        input: std::rc::Rc<std::cell::RefCell<egui::RawInput>>,
        output: std::rc::Rc<std::cell::RefCell<Option<egui::FullOutput>>>,
        builders: std::rc::Rc<std::cell::RefCell<Vec<egui::ViewportBuilder>>>,
    }
    impl NativeSettings {
        fn new(ctx: &egui::Context) -> Self {
            Self::window(
                ctx,
                crate::settings_window::id(),
                "Benis Einstellungen",
                egui::vec2(520.0, 680.0),
            )
        }
        fn window(
            ctx: &egui::Context,
            id: egui::ViewportId,
            title: &'static str,
            size: Vec2,
        ) -> Self {
            use std::{cell::RefCell, rc::Rc};
            let rig = Self {
                input: Rc::new(RefCell::new(egui::RawInput::default())),
                output: Rc::new(RefCell::new(None)),
                builders: Rc::new(RefCell::new(vec![])),
            };
            ctx.set_embed_viewports(false);
            ctx.enable_accesskit();
            ctx.all_styles_mut(|style| style.animation_time = 0.0);
            let input = rig.input.clone();
            let output = rig.output.clone();
            let builders = rig.builders.clone();
            egui::Context::set_immediate_viewport_renderer(move |ctx, mut viewport| {
                assert_eq!(viewport.ids.this, id);
                assert_eq!(viewport.ids.parent, egui::ViewportId::ROOT);
                builders.borrow_mut().push(viewport.builder.clone());
                let mut raw = input.borrow().clone();
                raw.viewport_id = viewport.ids.this;
                raw.screen_rect.get_or_insert(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    viewport.builder.inner_size.unwrap_or(size),
                ));
                let info = raw.viewports.entry(viewport.ids.this).or_default();
                info.parent = Some(viewport.ids.parent);
                info.title = Some(title.into());
                info.inner_rect = raw.screen_rect;
                let mut child = ctx.run_ui(raw, |ui| (viewport.viewport_ui_cb)(ui));
                child.textures_delta.clear();
                output.borrow_mut().replace(child);
            });
            rig
        }
    }

    #[test]
    fn avatar_click_opens_an_attached_bubble_at_every_size_without_a_chat_viewport() {
        for size in BuddySize::ALL {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = app(&ctx);
            app.size = size;
            app.player.paused = true;
            app.message_until = Instant::now();
            let output = frame(&ctx, &mut app, 0.0, vec![]);
            let avatar = accesskit_node(&output, "Beni: Menü-Rad öffnen").0;
            let original = avatar_rect(&output, &app);
            output.drop_without_applying_deltas();
            let output = frame(&ctx, &mut app, 0.1, vec![accessible_click(avatar)]);
            let talk = accesskit_node(&output, "Mit Beni reden").0;
            output.drop_without_applying_deltas();
            let output = frame(&ctx, &mut app, 0.2, vec![accessible_click(talk)]);
            assert!(app.chat.open && !app.controls_open);
            output.drop_without_applying_deltas();
            let dimensions = crate::chat::dimensions(size.dimensions(), true, None);
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, dimensions)),
                    time: Some(0.3),
                    ..Default::default()
                },
                |ui| app.show_ui(ui),
            );
            output.textures_delta.clear();
            assert_eq!(output.viewport_output.len(), 1);
            let actual = avatar_rect(&output, &app);
            assert!((actual.size() - original.size()).length() < 0.01);
            assert!(actual.min.y >= crate::chat::EXTRA_HEIGHT);
            let bounds = Rect::from_min_size(egui::Pos2::ZERO, dimensions);
            for label in [
                "Lokales Ollama verbinden",
                "Senden",
                "Abbrechen",
                "Gespräch leeren",
            ] {
                let rect = text_rect(&output, label);
                assert!(bounds.contains_rect(rect));
                assert!(app.hit_map.contains(rect.center()));
            }
            let close = accesskit_node(&output, "Gespräch schliessen").0;
            output.drop_without_applying_deltas();
            let (_, token) = app
                .chat
                .session
                .start("test", "private question", 0)
                .unwrap();
            let output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(bounds),
                    time: Some(0.4),
                    events: vec![accessible_click(close)],
                    ..Default::default()
                },
                |ui| app.show_ui(ui),
            );
            assert!(!app.chat.open && token.is_cancelled());
            assert!(
                !output.viewport_output[&egui::ViewportId::ROOT]
                    .commands
                    .contains(&egui::ViewportCommand::Close)
            );
            output.drop_without_applying_deltas();
        }
    }

    #[test]
    fn emotions_use_existing_one_shots_without_restarting_and_restore_pause() {
        for paused in [false, true] {
            let ctx = egui::Context::default();
            let mut app = app(&ctx);
            app.player.paused = paused;
            app.settings.reduced_motion = true;
            let original = app.player.clone();
            app.open_chat(&ctx);
            let (request, _) = app.chat.session.start("test", "Frage", 0).unwrap();
            let reply=buddy_chat::Reply::decode(r#"{"v":1,"segments":[{"text":"Hallo!","emotion":"friendly"},{"text":"Eine gute Idee!","emotion":"happy"}]}"#).unwrap();
            app.chat
                .receive(request.id(), buddy_chat::StreamEvent::Reply(reply), 1);
            app.chat.receive(
                request.id(),
                buddy_chat::StreamEvent::Finished {
                    reason: buddy_chat::FinishReason::Stop,
                    usage: buddy_chat::Usage::default(),
                },
                2,
            );
            app.sync_chat_animation(Instant::now());
            assert_eq!(app.player.state(), "hallo");
            assert_eq!(app.player.paused, paused);
            if !paused {
                app.player.advance(&app.engine, 60_000.0, 1.0).unwrap();
                let after = app.player.state().to_owned();
                app.sync_chat_animation(Instant::now());
                assert_eq!(
                    app.player.state(),
                    after,
                    "same cue must not restart its one-shot"
                );
            }
            app.chat.next_part();
            app.sync_chat_animation(Instant::now());
            assert_eq!(app.player.state(), "jubeln");
            assert_eq!(app.player.paused, paused);
            app.chat.close();
            app.sync_chat_animation(Instant::now());
            assert_eq!(app.player.state(), original.state());
            assert_eq!(app.player.paused, paused);
            assert!(app.settings.reduced_motion);
            for emotion in buddy_chat::Emotion::ALL {
                assert!(app.engine.animations.contains_key(emotion.animation()));
            }
        }
    }

    #[test]
    fn conversation_geometry_preserves_the_avatar_anchor_and_reachable_controls() {
        for size in BuddySize::ALL {
            for monitor in [egui::vec2(1440.0, 900.0), egui::vec2(1024.0, 768.0)] {
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                let mut app = app(&ctx);
                app.size = size;
                app.open_chat(&ctx);
                let base = size.dimensions();
                let desired =
                    crate::chat::dimensions_with_space(base, true, Some(monitor), Some(16.0));
                let origin = egui::pos2(700.0, 100.0);
                let expanded = origin - crate::chat::offset(base, desired);
                let mut raw = egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, desired)),
                    ..Default::default()
                };
                raw.viewports
                    .entry(egui::ViewportId::ROOT)
                    .or_default()
                    .outer_rect = Some(Rect::from_min_size(expanded, desired));
                raw.viewports
                    .entry(egui::ViewportId::ROOT)
                    .or_default()
                    .monitor_size = Some(monitor);
                app.initial_geometry = false;
                app.chat_layout_open = true;
                app.size_pending = false;
                let mut output = ctx.run_ui(raw, |ui| app.show_ui(ui));
                output.textures_delta.clear();
                assert_eq!(app.rest_position, Some([origin.x, origin.y]));
                let bounds = Rect::from_min_size(egui::Pos2::ZERO, desired);
                for label in ["Senden", "Abbrechen", "Gespräch leeren"] {
                    assert!(bounds.contains_rect(text_rect(&output, label)));
                }
                app.chat.close();
                app.sync_conversation_geometry(&ctx, None);
                assert_eq!(crate::chat::dimensions(base, false, Some(monitor)), base);
            }
        }
    }

    #[test]
    fn explicit_conversation_takes_over_a_status_bubble_without_sending_its_text() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.player.play(&app.engine, "daumen", true).unwrap();
        app.player.paused = true;
        app.active_status = true;
        app.message = "private-status".into();
        app.message_until = Instant::now() + Duration::from_secs(30);
        app.open_chat(&ctx);
        assert!(!app.active_status && app.message_until <= Instant::now());
        assert_eq!(app.player.state(), "warten");
        assert!(app.player.paused);
        let (request, _) = app.chat.session.start("test", "Frage", 0).unwrap();
        app.sync_chat_animation(Instant::now());
        assert_eq!(app.player.state(), "nachdenken");
        assert!(
            !request
                .messages()
                .iter()
                .any(|m| m.text().contains("private-status"))
        );
    }

    #[test]
    fn changing_to_a_smaller_monitor_reflows_the_open_conversation() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.size = BuddySize::Large;
        app.open_chat(&ctx);
        app.chat_layout_open = true;
        app.size_pending = false;
        app.initial_geometry = false;
        app.last_monitor_size = Some(egui::vec2(1440.0, 1200.0));
        app.space_above = Some(16.0);
        let mut input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(420.0, 900.0),
            )),
            ..Default::default()
        };
        input
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .monitor_size = Some(egui::vec2(1024.0, 768.0));
        let mut output = ctx.run_ui(input, |ui| app.show_ui(ui));
        output.textures_delta.clear();
        assert!(
            output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .contains(&egui::ViewportCommand::InnerSize(egui::vec2(860.0, 560.0)))
        );
    }

    #[test]
    fn chat_uses_original_gestures_and_restores_pause_and_keeps_status_history_out_of_storage() {
        for paused in [false, true] {
            let ctx = egui::Context::default();
            let mut app = app(&ctx);
            app.player.paused = paused;
            app.settings.reduced_motion = true;
            let original = app.player.clone();
            app.open_chat(&ctx);
            let (request, _) = app
                .chat
                .session
                .start("test", "private-chat-question", 0)
                .unwrap();
            app.sync_chat_animation(Instant::now());
            assert_eq!(app.player.state(), "nachdenken");
            assert_eq!(app.player.paused, paused);
            app.chat.session.receive(
                request.id(),
                buddy_chat::StreamEvent::Delta("private-chat-answer".into()),
                1,
            );
            app.sync_chat_animation(Instant::now());
            assert_eq!(app.player.state(), "reden");
            assert_eq!(app.player.paused, paused);
            app.chat.session.receive(
                request.id(),
                buddy_chat::StreamEvent::Finished {
                    reason: buddy_chat::FinishReason::Stop,
                    usage: buddy_chat::Usage::default(),
                },
                2,
            );
            app.sync_chat_animation(Instant::now());
            assert_eq!(app.player.state(), original.state());
            assert_eq!(app.player.paused, paused);
            let mut storage = TestStorage::default();
            eframe::App::save(&mut app, &mut storage);
            assert!(storage.0.values().all(
                |v| !v.contains("private-chat-question") && !v.contains("private-chat-answer")
            ));
            assert_eq!(app.history.len(), 0);
        }
    }

    #[test]
    fn native_settings_have_their_own_surface_without_covering_or_resizing_beni() {
        let ctx = egui::Context::default();
        let rig = NativeSettings::new(&ctx);
        let mut app = app(&ctx);
        app.size = BuddySize::Small;
        app.player.paused = true;
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        let avatar = avatar_rect(&output, &app);
        output.drop_without_applying_deltas();
        app.open_settings(&ctx);
        frame(&ctx, &mut app, 0.1, vec![]).drop_without_applying_deltas();
        let mut output = frame(&ctx, &mut app, 0.2, vec![]);
        output.textures_delta.clear();
        assert_eq!(avatar_rect(&output, &app), avatar);
        assert!(
            !output
                .shapes
                .iter()
                .any(|s| find_text(&s.shape, "Weniger Bewegung").is_some())
        );
        assert!(app.hit_map.contains(avatar.center()));
        let builders = rig.builders.borrow();
        let builder = builders.last().unwrap();
        assert_eq!(builder.inner_size, Some(egui::vec2(520.0, 680.0)));
        assert_eq!(builder.transparent, Some(false));
        assert_eq!(builder.mouse_passthrough, Some(false));
        assert_eq!(builder.decorations, Some(true));
        assert_eq!(builder.resizable, Some(true));
        assert_eq!(builder.window_level, Some(egui::WindowLevel::Normal));
        let child = rig.output.borrow();
        let child = child.as_ref().unwrap();
        let close = text_rect(child, "Schliessen");
        assert!(
            Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(520.0, 680.0)).contains_rect(close)
        );
        assert!(child.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Rect(r) if r.fill.a() == 255 && r.rect.contains_rect(close))));
        assert_eq!(
            accesskit_node(child, "Weniger Bewegung").1.role(),
            egui::accesskit::Role::CheckBox
        );
        assert_eq!(
            accesskit_node(child, "Benis Nachricht").1.role(),
            egui::accesskit::Role::TextInput
        );
    }

    #[test]
    fn native_settings_minimum_size_keeps_the_close_button_outside_scrolling_content() {
        let ctx = egui::Context::default();
        let rig = NativeSettings::new(&ctx);
        let screen = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(380.0, 360.0));
        rig.input.borrow_mut().screen_rect = Some(screen);
        let mut app = app(&ctx);
        app.menu_open = true;
        for index in 0..3 {
            frame(&ctx, &mut app, index as f64 * 0.1, vec![]).drop_without_applying_deltas();
        }
        let child = rig.output.borrow();
        assert!(screen.contains_rect(text_rect(child.as_ref().unwrap(), "Schliessen")));
    }

    #[test]
    fn native_settings_commands_resize_and_relevel_buddy_and_close_only_settings() {
        let ctx = egui::Context::default();
        let rig = NativeSettings::new(&ctx);
        let mut app = app(&ctx);
        app.player.paused = true;
        app.controls_open = true;
        app.open_settings(&ctx);
        for index in 0..3 {
            frame(&ctx, &mut app, index as f64 * 0.1, vec![]).drop_without_applying_deltas();
        }
        let child = rig.output.borrow();
        let large = accesskit_node(child.as_ref().unwrap(), "Gross").0;
        let top = accesskit_node(child.as_ref().unwrap(), "Immer im Vordergrund").0;
        drop(child);
        rig.input.borrow_mut().events = vec![accessible_click(large), accessible_click(top)];
        let mut output = frame(&ctx, &mut app, 0.4, vec![]);
        output.textures_delta.clear();
        assert_eq!(app.size, BuddySize::Large);
        let commands = &output.viewport_output[&egui::ViewportId::ROOT].commands;
        assert!(commands.contains(&egui::ViewportCommand::WindowLevel(
            egui::WindowLevel::Normal
        )));
        assert!(commands.contains(&egui::ViewportCommand::InnerSize(
            BuddySize::Large.dimensions()
        )));
        assert!(
            !rig.output.borrow().as_ref().unwrap().viewport_output[&crate::settings_window::id()]
                .commands
                .iter()
                .any(|c| matches!(
                    c,
                    egui::ViewportCommand::WindowLevel(_) | egui::ViewportCommand::InnerSize(_)
                ))
        );
        rig.input.borrow_mut().events.clear();
        rig.input
            .borrow_mut()
            .viewports
            .entry(crate::settings_window::id())
            .or_default()
            .events = vec![egui::ViewportEvent::Close];
        let mut output = frame(&ctx, &mut app, 0.5, vec![]);
        output.textures_delta.clear();
        assert!(!app.menu_open);
        assert!(app.controls_open && app.player.paused);
        assert!(
            !output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .contains(&egui::ViewportCommand::Close)
        );
    }

    #[test]
    fn native_settings_escape_finishes_editing_before_closing_its_window() {
        let ctx = egui::Context::default();
        let rig = NativeSettings::new(&ctx);
        let mut app = app(&ctx);
        app.menu_open = true;
        for index in 0..3 {
            frame(&ctx, &mut app, index as f64 * 0.1, vec![]).drop_without_applying_deltas();
        }
        let input = accesskit_node(rig.output.borrow().as_ref().unwrap(), "Benis Nachricht").0;
        rig.input.borrow_mut().events = vec![egui::Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Focus,
                target_tree: egui::accesskit::TreeId::ROOT,
                target_node: input,
                data: None,
            },
        )];
        frame(&ctx, &mut app, 0.3, vec![]).drop_without_applying_deltas();
        rig.input.borrow_mut().events = vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }];
        frame(&ctx, &mut app, 0.4, vec![]).drop_without_applying_deltas();
        assert!(app.menu_open, "Escape first leaves the text field");
        rig.input.borrow_mut().events.clear();
        frame(&ctx, &mut app, 0.5, vec![]).drop_without_applying_deltas();
        rig.input.borrow_mut().events = vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }];
        let mut output = frame(&ctx, &mut app, 0.6, vec![]);
        output.textures_delta.clear();
        assert!(!app.menu_open);
        assert!(
            !output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .contains(&egui::ViewportCommand::Close)
        );
    }

    #[test]
    fn open_settings_do_not_block_incoming_status_or_close_when_buddy_controls_collapse() {
        let endpoint = common::TestEndpoint::new();
        let ctx = egui::Context::default();
        let _rig = NativeSettings::new(&ctx);
        let mut app = app(&ctx);
        app.server = Some(buddy_ipc::Server::start_at(endpoint.endpoint.clone(), || {}).unwrap());
        app.menu_open = true;
        buddy_ipc::notify(
            &endpoint.endpoint,
            buddy_core::notifications::Notification::new("Build grün"),
        )
        .unwrap();
        app.deliver_status(true, Instant::now());
        assert!(app.active_status && app.menu_open);
        assert_eq!(app.message, "Build grün");
        app.controls_open = true;
        app.player.paused = true;
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        let avatar = avatar_rect(&output, &app);
        output.drop_without_applying_deltas();
        click(&ctx, &mut app, avatar.center(), 0.1).drop_without_applying_deltas();
        assert!(!app.controls_open);
        assert!(app.menu_open && app.player.paused);
    }
    #[test]
    fn local_timer_records_and_displays_once_without_ipc_and_preserves_animation_pause() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        let now = Instant::now();
        app.player.paused = true;
        app.settings.reduced_motion = true;
        app.timer.start(crate::timer::Phase::Focus, now);
        let deadline = now + Duration::from_secs(1500);
        app.advance_timer(deadline);
        app.advance_timer(deadline + Duration::from_secs(1));
        assert_eq!(app.history.len(), 1);
        assert!(app.server.is_none() && !app.ipc_enabled);
        app.deliver_status(true, deadline);
        assert_eq!(app.message, crate::timer::Phase::Focus.message());
        assert_eq!(app.player.state(), "daumen");
        assert!(app.player.paused && app.active_status);
        assert_eq!(app.message_source.as_deref(), Some("timer"));
        app.dismiss_speech(deadline);
        let expired = app.message_until;
        app.deliver_status(true, deadline + Duration::from_secs(1));
        assert_eq!(app.message_until, expired, "no repeated completion");
        app.set_do_not_disturb(true, deadline);
        assert!(app.player.paused);
    }

    #[test]
    fn timer_notices_wait_for_visibility_quiet_history_errors_and_existing_speech() {
        for blocked in ["hidden", "quiet", "history", "error", "speech", "status"] {
            let ctx = egui::Context::default();
            let mut app = app(&ctx);
            let now = Instant::now();
            app.timer.start(crate::timer::Phase::Break, now);
            let deadline = now + Duration::from_secs(300);
            app.message_until = now;
            app.advance_timer(deadline);
            match blocked {
                "quiet" => app.set_do_not_disturb(true, deadline),
                "history" => app.open_history(),
                "error" => app.report_error("Hinweis".into(), deadline),
                "speech" => app.message_until = deadline + Duration::from_secs(30),
                "status" => {
                    app.message_until = deadline + Duration::from_secs(30);
                    app.active_status = true;
                }
                _ => {}
            }
            let message = app.message.clone();
            app.deliver_status(blocked != "hidden", deadline);
            assert_eq!(app.message, message, "{blocked}");
            assert_eq!(app.history.len(), 1, "{blocked}");
            app.set_do_not_disturb(false, deadline);
            app.close_history();
            app.expire_error(deadline + Duration::from_secs(10));
            app.dismiss_speech(deadline);
            app.deliver_status(true, deadline + Duration::from_secs(10));
            assert_eq!(
                app.message,
                crate::timer::Phase::Break.message(),
                "{blocked}"
            );
            assert_eq!(app.player.state(), "idee");
        }
    }

    #[test]
    fn queued_ipc_precedes_timer_and_stale_timer_remains_only_in_history() {
        let endpoint = common::TestEndpoint::new();
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.server = Some(buddy_ipc::Server::start_at(endpoint.endpoint.clone(), || {}).unwrap());
        buddy_ipc::notify(
            &endpoint.endpoint,
            buddy_core::notifications::Notification::new("Build grün vor Timer"),
        )
        .unwrap();
        let now = Instant::now();
        // The timer's end is now; IPC keeps its real transport clock and TTL.
        app.timer
            .start(crate::timer::Phase::Break, now - Duration::from_secs(300));
        app.collect_history();
        app.advance_timer(now);
        app.message_until = now;
        app.deliver_status(true, now);
        assert_eq!(app.message, "Build grün vor Timer");
        assert_eq!(app.history.len(), 2);
        app.deliver_status(true, now + Duration::from_secs(1));
        assert_eq!(app.message, "Build grün vor Timer");
        app.dismiss_speech(now);
        app.deliver_status(true, now + Duration::from_secs(120));
        assert_eq!(
            app.message, "Build grün vor Timer",
            "expired timer cannot replace it"
        );
        assert_eq!(app.history.len(), 2);
        app.history.clear();
        assert_eq!(app.history.len(), 0);
    }

    #[test]
    fn native_timer_controls_start_pause_resume_and_cancel_without_changing_avatar_pause() {
        let ctx = egui::Context::default();
        let rig = NativeSettings::new(&ctx);
        let mut app = app(&ctx);
        app.open_settings(&ctx);
        app.player.paused = true;
        for (index, label) in [
            "Fokus & Pause",
            "Fokus starten",
            "Timer unterbrechen",
            "Timer fortsetzen",
            "Timer abbrechen",
            "Pause starten",
            "Timer abbrechen",
        ]
        .into_iter()
        .enumerate()
        {
            frame(&ctx, &mut app, index as f64, vec![]).drop_without_applying_deltas();
            let event = {
                let output = rig.output.borrow();
                let output = output.as_ref().unwrap();
                accessible_click(accesskit_node(output, label).0)
            };
            rig.input.borrow_mut().events = vec![event];
            frame(&ctx, &mut app, index as f64 + 0.1, vec![]).drop_without_applying_deltas();
            rig.input.borrow_mut().events.clear();
            assert!(app.player.paused && app.menu_open);
        }
        assert!(
            app.timer
                .tick(Instant::now() + Duration::from_secs(10000))
                .is_none()
        );
        assert_eq!(
            app.history.len(),
            0,
            "cancelled sessions do not create completions"
        );
    }

    #[test]
    fn timer_advances_in_logic_without_ui_and_keeps_static_housekeeping_budget() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.hidden = true;
        app.settings.reduced_motion = true;
        let now = Instant::now();
        app.timer
            .start(crate::timer::Phase::Break, now - Duration::from_secs(300));
        let mut native_frame = eframe::Frame::_new_kittest();
        let _ = ctx.run_logic(&egui::RawInput::default(), |ctx| {
            eframe::App::logic(&mut app, ctx, &mut native_frame);
        });
        assert_eq!(app.history.len(), 1);
        assert!(!app.active_status);
        assert_eq!(app.repaint_delay(now, false), Duration::from_secs(1));
        assert_eq!(app.repaint_delay(now, true), Duration::from_secs(1));
        app.hidden = false;
        app.message_until = now;
        app.deliver_status(true, now);
        assert_eq!(app.message, crate::timer::Phase::Break.message());
    }
    #[test]
    fn reminders_finish_without_ipc_and_leave_the_focus_timer_and_avatar_pause_running() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        let now = Instant::now();
        app.player.paused = true;
        app.timer.start(crate::timer::Phase::Focus, now);
        app.reminders.schedule("Wasser trinken", 1, now).unwrap();
        app.advance_reminders(now + Duration::from_secs(60));
        app.advance_reminders(now + Duration::from_secs(61));
        assert_eq!(app.history.len(), 1);
        assert!(app.server.is_none() && !app.ipc_enabled);
        app.message_until = now;
        app.deliver_status(true, now + Duration::from_secs(60));
        assert_eq!(app.message, "Wasser trinken");
        assert_eq!(app.message_source.as_deref(), Some("erinnerung"));
        assert_eq!(app.player.state(), "idee");
        assert!(app.player.paused && app.active_status);
        assert!(app.timer.tick(now + Duration::from_secs(60)).is_none());
        assert!(app.timer.tick(now + Duration::from_secs(1500)).is_some());
    }

    #[test]
    fn reminders_respect_all_delivery_hindrances_and_keep_history() {
        for blocked in ["hidden", "quiet", "history", "error", "speech", "status"] {
            let ctx = egui::Context::default();
            let mut app = app(&ctx);
            let now = Instant::now();
            app.reminders
                .schedule("Kurz aufstehen", 1, now - Duration::from_secs(60))
                .unwrap();
            app.advance_reminders(now);
            app.message_until = now;
            match blocked {
                "quiet" => app.set_do_not_disturb(true, now),
                "history" => app.open_history(),
                "error" => app.report_error("Hinweis".into(), now),
                "speech" => app.message_until = now + Duration::from_secs(30),
                "status" => {
                    app.message_until = now + Duration::from_secs(30);
                    app.active_status = true;
                }
                _ => {}
            }
            let before = app.message.clone();
            app.deliver_status(blocked != "hidden", now);
            assert_eq!(app.message, before, "{blocked}");
            assert_eq!(app.history.len(), 1);
            app.set_do_not_disturb(false, now);
            app.close_history();
            app.expire_error(now + Duration::from_secs(10));
            app.dismiss_speech(now);
            app.deliver_status(true, now + Duration::from_secs(10));
            assert_eq!(app.message, "Kurz aufstehen", "{blocked}");
        }
    }

    #[test]
    fn conversation_escape_releases_queued_notices_in_order_and_never_resumes_its_request() {
        let endpoint = common::TestEndpoint::new();
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.server = Some(buddy_ipc::Server::start_at(endpoint.endpoint.clone(), || {}).unwrap());
        app.player.paused = true;
        app.open_chat(&ctx);
        let (_, cancellation) = app.chat.session.start("test", "Frage", 0).unwrap();
        let now = Instant::now();
        app.sync_chat_animation(now);
        assert_eq!(app.player.state(), "nachdenken");
        app.timer
            .start(crate::timer::Phase::Break, now - Duration::from_secs(300));
        app.reminders
            .schedule("Kurz aufstehen", 1, now - Duration::from_secs(60))
            .unwrap();
        app.advance_timer(now);
        app.advance_reminders(now);
        buddy_ipc::notify(
            &endpoint.endpoint,
            buddy_core::notifications::Notification::new("Build grün"),
        )
        .unwrap();
        app.deliver_status(true, now);
        assert!(!app.active_status);
        assert_eq!(app.server.as_ref().unwrap().pending(), 1);
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
        assert!(!app.chat.open);
        assert!(cancellation.is_cancelled());
        app.sync_chat_animation(now);
        assert!(app.chat_player.is_none());
        for text in [
            "Build grün",
            crate::timer::Phase::Break.message(),
            "Kurz aufstehen",
        ] {
            app.dismiss_speech(now);
            app.deliver_status(true, now);
            assert_eq!(app.message, text);
            assert!(app.player.paused);
            let deadline = app.message_until;
            app.deliver_status(true, now);
            assert_eq!(
                app.message_until, deadline,
                "an active notice must not be preempted"
            );
        }
        app.dismiss_speech(now);
        app.deliver_status(true, now);
        assert!(!app.active_status);
    }

    #[test]
    fn ipc_then_timer_then_reminders_deliver_in_order_without_repeating() {
        let endpoint = common::TestEndpoint::new();
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.server = Some(buddy_ipc::Server::start_at(endpoint.endpoint.clone(), || {}).unwrap());
        buddy_ipc::notify(
            &endpoint.endpoint,
            buddy_core::notifications::Notification::new("Build grün"),
        )
        .unwrap();
        let now = Instant::now();
        app.timer
            .start(crate::timer::Phase::Break, now - Duration::from_secs(300));
        app.reminders
            .schedule("Erste Erinnerung", 1, now - Duration::from_secs(60))
            .unwrap();
        app.reminders
            .schedule("Zweite Erinnerung", 1, now - Duration::from_secs(60))
            .unwrap();
        app.collect_history();
        app.advance_timer(now);
        app.advance_reminders(now);
        assert_eq!(app.history.len(), 4);
        for text in [
            "Build grün",
            crate::timer::Phase::Break.message(),
            "Erste Erinnerung",
            "Zweite Erinnerung",
        ] {
            app.dismiss_speech(now);
            app.deliver_status(true, now);
            assert_eq!(app.message, text);
        }
        app.dismiss_speech(now);
        let deadline = app.message_until;
        app.deliver_status(true, now);
        assert_eq!(app.message_until, deadline);
    }

    #[test]
    fn reminders_tick_while_hidden_expire_bubbles_and_never_save_text_or_schedule() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.hidden = true;
        app.settings.reduced_motion = true;
        let now = Instant::now();
        app.reminders
            .schedule(
                "Sitzungsprivater Testtext",
                1,
                now - Duration::from_secs(180),
            )
            .unwrap();
        app.reminders
            .schedule("Noch geplanter privater Text", 1440, now)
            .unwrap();
        let mut native_frame = eframe::Frame::_new_kittest();
        let _ = ctx.run_logic(&egui::RawInput::default(), |ctx| {
            eframe::App::logic(&mut app, ctx, &mut native_frame)
        });
        assert_eq!(app.history.len(), 1);
        assert_eq!(app.repaint_delay(now, false), Duration::from_secs(1));
        app.hidden = false;
        app.message_until = now;
        let text = app.message.clone();
        app.deliver_status(true, now);
        assert_eq!(app.message, text);
        let mut storage = TestStorage::default();
        eframe::App::save(&mut app, &mut storage);
        assert!(storage.0.keys().all(|k| !k.contains("reminder")));
        assert!(
            storage
                .0
                .values()
                .all(|v| !v.contains("privater") && !v.contains("Sitzungsprivater"))
        );
        app.history.clear();
        assert_eq!(app.history.len(), 0);
    }

    #[test]
    fn native_reminder_editor_plans_and_removes_and_minimum_size_keeps_footer_visible() {
        let ctx = egui::Context::default();
        let rig = NativeSettings::new(&ctx);
        let mut app = app(&ctx);
        app.open_settings(&ctx);
        frame(&ctx, &mut app, 0.0, vec![]).drop_without_applying_deltas();
        let header = accesskit_node(rig.output.borrow().as_ref().unwrap(), "Erinnerungen").0;
        rig.input.borrow_mut().events = vec![accessible_click(header)];
        frame(&ctx, &mut app, 0.1, vec![]).drop_without_applying_deltas();
        let input = accesskit_node(rig.output.borrow().as_ref().unwrap(), "Erinnerungstext").0;
        rig.input.borrow_mut().events = vec![egui::Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Focus,
                target_tree: egui::accesskit::TreeId::ROOT,
                target_node: input,
                data: None,
            },
        )];
        frame(&ctx, &mut app, 0.2, vec![]).drop_without_applying_deltas();
        rig.input.borrow_mut().events = vec![egui::Event::Text("Wasser trinken".into())];
        frame(&ctx, &mut app, 0.25, vec![]).drop_without_applying_deltas();
        assert_eq!(
            accesskit_node(rig.output.borrow().as_ref().unwrap(), "Erinnerungstext")
                .1
                .value(),
            Some("Wasser trinken")
        );
        let add = accesskit_node(rig.output.borrow().as_ref().unwrap(), "Erinnerung planen").0;
        rig.input.borrow_mut().events = vec![accessible_click(add)];
        frame(&ctx, &mut app, 0.3, vec![]).drop_without_applying_deltas();
        rig.input.borrow_mut().events.clear();
        frame(&ctx, &mut app, 0.4, vec![]).drop_without_applying_deltas();
        let remove = accesskit_node(
            rig.output.borrow().as_ref().unwrap(),
            "Erinnerung entfernen: Wasser trinken",
        )
        .0;
        rig.input.borrow_mut().events = vec![accessible_click(remove)];
        frame(&ctx, &mut app, 0.5, vec![]).drop_without_applying_deltas();
        assert!(
            app.reminders
                .tick(Instant::now() + Duration::from_secs(600))
                .is_empty()
        );
        for _ in 0..10 {
            app.reminders
                .schedule(&"Langer Text ".repeat(20), 1, Instant::now())
                .unwrap();
        }
        rig.input.borrow_mut().events.clear();
        rig.input.borrow_mut().screen_rect = Some(Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(380.0, 360.0),
        ));
        frame(&ctx, &mut app, 0.6, vec![]).drop_without_applying_deltas();
        let output = rig.output.borrow();
        let close = text_rect(output.as_ref().unwrap(), "Schliessen");
        assert!(
            Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(380.0, 360.0)).contains_rect(close)
        );
        assert!(app.menu_open && app.history.len() == 0);
    }
    #[test]
    fn quiet_idle_reuses_existing_gestures_without_speech_and_yields_to_manual_actions() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = app(&ctx);
        app.message_until = Instant::now();
        let original_message = app.message.clone();
        let deadline = app.message_until;
        for index in 0..18 {
            app.last_tick = Instant::now() - Duration::from_secs(1);
            frame(&ctx, &mut app, index as f64, vec![]).drop_without_applying_deltas();
        }
        assert_eq!(app.player.state(), "nachdenken");
        assert!(app.idle.owns(&app.player));
        assert_eq!(
            app.repaint_delay(Instant::now(), true),
            Duration::from_millis(66)
        );
        assert_eq!(app.message, original_message);
        assert_eq!(app.message_until, deadline);
        assert!(!app.active_status);
        let mut output = frame(&ctx, &mut app, 18.0, vec![]);
        output.textures_delta.clear();
        assert_eq!(
            accesskit_node(&output, "Beni: Menü-Rad öffnen")
                .1
                .description(),
            Some("Animation: Denken")
        );
        app.play("daumen", "Build fertig");
        assert!(!app.idle.owns(&app.player));
        assert_eq!(app.player.state(), "daumen");
        assert_eq!(app.message, "Build fertig");
        assert_eq!(
            app.repaint_delay(Instant::now(), true),
            Duration::from_millis(33)
        );
    }

    #[test]
    fn idle_gestures_respect_motion_pause_controls_hidden_speech_and_errors() {
        for blocked in [
            "manual-reduction",
            "system-reduction",
            "pause",
            "menu",
            "controls",
            "hidden",
            "speech",
            "error",
        ] {
            let ctx = egui::Context::default();
            let mut app = app(&ctx);
            app.message_until = Instant::now();
            match blocked {
                "manual-reduction" => app.settings.reduced_motion = true,
                "system-reduction" => app.motion.system = Some(true),
                "pause" => app.player.paused = true,
                "menu" => app.menu_open = true,
                "controls" => app.controls_open = true,
                "hidden" => app.hidden = true,
                "speech" => app.message_until = Instant::now() + Duration::from_secs(60),
                "error" => app.report_error("Hinweis".into(), Instant::now()),
                _ => unreachable!(),
            }
            for index in 0..20 {
                app.last_tick = Instant::now() - Duration::from_secs(1);
                frame(&ctx, &mut app, index as f64, vec![]).drop_without_applying_deltas();
            }
            assert_eq!(app.player.state(), "warten", "blocked by {blocked}");
        }
    }

    #[test]
    fn reading_history_suspends_and_restores_an_idle_gesture_and_its_pause() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.message_until = Instant::now();
        app.idle
            .advance(&app.engine, &mut app.player, 18_000.0, 1.0, true)
            .unwrap();
        app.player.advance(&app.engine, 500.0, 1.0).unwrap();
        app.open_history();
        for index in 0..5 {
            app.last_tick = Instant::now() - Duration::from_secs(1);
            frame(&ctx, &mut app, index as f64, vec![]).drop_without_applying_deltas();
        }
        app.player.paused = true;
        app.close_history();
        assert_eq!(app.player.state(), "nachdenken");
        assert_eq!(app.player.elapsed_ms(), 500.0);
        assert!(app.player.paused && app.idle.owns(&app.player));
        assert_eq!(
            app.repaint_delay(Instant::now(), true),
            Duration::from_secs(1)
        );
        app.player.paused = false;
        app.player.advance(&app.engine, 10_000.0, 1.0).unwrap();
        app.idle
            .advance(&app.engine, &mut app.player, 0.0, 1.0, true)
            .unwrap();
        app.idle
            .advance(&app.engine, &mut app.player, 18_000.0, 1.0, true)
            .unwrap();
        assert_eq!(
            app.player.state(),
            "drehen",
            "resume the existing playlist rather than start it again"
        );
    }

    #[test]
    fn system_motion_changes_pose_and_budget_without_changing_manual_settings_or_playback() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.player.play(&app.engine, "denken", false).unwrap();
        app.player.advance(&app.engine, 537.0, 1.0).unwrap();
        let elapsed = app.player.elapsed_ms();
        let deadline = app.message_until;
        let stage = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(320.0, 300.0));
        let positions = |app: &BuddyApp| {
            app.avatar_mesh(stage)
                .unwrap()
                .vertices
                .iter()
                .map(|v| v.pos)
                .collect::<Vec<_>>()
        };
        let moving = positions(&app);
        app.motion.system = Some(true);
        let reduced = positions(&app);
        assert_ne!(moving, reduced);
        assert_eq!(
            app.repaint_delay(Instant::now(), true),
            Duration::from_secs(1)
        );
        assert!(!app.settings.reduced_motion);
        assert_eq!(app.player.state(), "nachdenken");
        assert_eq!(app.player.elapsed_ms(), elapsed);
        assert_eq!(app.message_until, deadline);
        assert!(!app.player.paused);
        app.motion.follow_system = false;
        assert_eq!(positions(&app), moving);
        assert_eq!(
            app.repaint_delay(Instant::now(), true),
            Duration::from_millis(33)
        );
        app.motion.follow_system = true;
        app.motion.system = Some(false);
        assert_eq!(positions(&app), moving);
        app.settings.reduced_motion = true;
        assert_eq!(positions(&app), reduced);
        app.motion.system = None;
        assert_eq!(
            positions(&app),
            reduced,
            "unavailable OS must preserve manual reduction"
        );
    }

    #[test]
    fn motion_preferences_migrate_and_save_without_persisting_system_state() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        let mut storage = TestStorage::default();
        assert!(crate::motion::MotionPreference::load(None).follow_system);
        for invalid in ["bad", "1", "\"false\""] {
            eframe::Storage::set_string(&mut storage, "follow-system-motion", invalid.into());
            assert!(crate::motion::MotionPreference::load(Some(&storage)).follow_system);
        }
        app.motion.system = Some(true);
        eframe::App::save(&mut app, &mut storage);
        let saved: Settings = eframe::get_value(&storage, "buddy-settings").unwrap();
        assert!(
            !saved.reduced_motion,
            "OS preference must never overwrite manual choice"
        );
        app.motion.follow_system = false;
        eframe::App::save(&mut app, &mut storage);
        let loaded = crate::motion::MotionPreference::load(Some(&storage));
        assert!(!loaded.follow_system);
        assert_eq!(loaded.system, None);
        assert!(loaded.reduced(true));
        assert!(!loaded.reduced(false));
    }

    #[test]
    fn system_motion_toggle_is_accessible_and_preserves_a_paused_history_at_every_size() {
        for size in BuddySize::ALL {
            let ctx = egui::Context::default();
            ctx.all_styles_mut(|style| style.animation_time = 0.0);
            let mut app = app(&ctx);
            app.size = size;
            app.motion.system = Some(true);
            app.controls_open = true;
            app.menu_open = true;
            ctx.enable_accesskit();
            frame(&ctx, &mut app, 0.0, vec![]).drop_without_applying_deltas();
            scroll_settings(&ctx, &mut app, 0.1).drop_without_applying_deltas();
            let mut output = frame(&ctx, &mut app, 0.2, vec![]);
            output.textures_delta.clear();
            assert_eq!(
                accesskit_node(&output, "Systemeinstellung beachten")
                    .1
                    .role(),
                egui::accesskit::Role::CheckBox
            );
            text_rect(&output, "System: Weniger Bewegung ist aktiv.");
            let pos = text_rect(&output, "Systemeinstellung beachten").center();
            output.drop_without_applying_deltas();
            click(&ctx, &mut app, pos, 0.3).drop_without_applying_deltas();
            assert!(!app.motion.follow_system);
            app.open_history();
            app.player.paused = true;
            app.motion.system = Some(false);
            let mut output = frame(&ctx, &mut app, 0.4, vec![]);
            output.textures_delta.clear();
            text_rect(&output, "Meldungsverlauf (0)");
            output.drop_without_applying_deltas();
            assert_eq!(app.player.state(), "nachdenken");
            assert!(app.player.paused && app.history_player.is_some());
        }
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
            egui::Shape::Text(text)
                if text
                    .galley
                    .job
                    .text
                    .strip_prefix('\n')
                    .unwrap_or(&text.galley.job.text)
                    == needle =>
            {
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
    fn scroll_settings(ctx: &egui::Context, app: &mut BuddyApp, time: f64) -> egui::FullOutput {
        frame(
            ctx,
            app,
            time,
            vec![
                egui::Event::PointerMoved(egui::pos2(80.0, 260.0)),
                egui::Event::MouseWheel {
                    phase: egui::TouchPhase::Move,
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -240.0),
                    modifiers: Default::default(),
                },
            ],
        )
        .drop_without_applying_deltas();
        // Smooth scrolling consumes the wheel over several UI passes.
        for index in 1..8 {
            frame(ctx, app, time + index as f64 * 0.01, vec![]).drop_without_applying_deltas();
        }
        frame(ctx, app, time + 0.08, vec![])
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
        for label in [
            "Fokus",
            "Erinnerung",
            "Verlauf",
            "Ruhe",
            "Reden",
            "Einstellungen",
        ] {
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
    fn update_save_waits_for_storage_and_keeps_ipc_alive_on_write_failure() {
        struct CheckedStorage {
            values: TestStorage,
            fail: bool,
        }
        impl eframe::Storage for CheckedStorage {
            fn get_string(&self, key: &str) -> Option<String> {
                self.values.get_string(key)
            }
            fn set_string(&mut self, key: &str, value: String) {
                self.values.set_string(key, value);
            }
            fn remove_string(&mut self, key: &str) {
                self.values.remove_string(key);
            }
            fn flush(&mut self) {
                panic!("update handoff must use the synchronous API");
            }
            fn flush_sync(&mut self) -> std::io::Result<()> {
                if self.fail {
                    Err(std::io::Error::other("fixture"))
                } else {
                    Ok(())
                }
            }
        }
        let endpoint = common::TestEndpoint::new();
        let mut app = app(&egui::Context::default());
        app.ipc_enabled = true;
        app.server = Some(buddy_ipc::Server::start_at(endpoint.endpoint.clone(), || {}).unwrap());
        let mut storage = CheckedStorage {
            values: TestStorage::default(),
            fail: true,
        };
        let (_, chat_token) = app
            .chat
            .session
            .start("test", "private-update-question", 0)
            .unwrap();
        assert!(app.save_for_update(&mut storage).is_err());
        assert!(app.chat.busy() && !chat_token.is_cancelled());
        assert!(app.server.is_some() && app.ipc_enabled);
        buddy_ipc::notify(
            &endpoint.endpoint,
            buddy_core::notifications::Notification::new("Empfang bleibt aktiv"),
        )
        .unwrap();
        storage.fail = false;
        app.save_for_update(&mut storage).unwrap();
        assert!(!app.chat.busy() && chat_token.is_cancelled());
        assert!(app.server.is_none() && app.ipc_enabled);
        assert_eq!(
            eframe::get_value::<bool>(&storage, "status-receiving"),
            Some(true)
        );
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
        app.active_status = true;
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
            assert!(app.active_status && app.player.paused);
            output.drop_without_applying_deltas();
        }
    }
    #[test]
    fn wheel_opens_direct_focus_and_reminder_pages_and_keeps_live_speech_and_pause() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let rig = NativeSettings::new(&ctx);
        let mut app = app(&ctx);
        app.player.paused = true;
        app.active_status = true;
        let deadline = app.message_until;
        for (action, page, required, absent) in [
            (
                crate::action_wheel::Action::Focus,
                crate::settings_window::Page::Focus,
                "Fokus starten",
                "Erinnerungstext",
            ),
            (
                crate::action_wheel::Action::Reminder,
                crate::settings_window::Page::Reminders,
                "Erinnerungstext",
                "Fokus starten",
            ),
        ] {
            app.controls_open = true;
            let output = frame(&ctx, &mut app, 0.0, vec![]);
            let button = accesskit_node(&output, action.accessible_label(false)).0;
            output.drop_without_applying_deltas();
            frame(&ctx, &mut app, 0.1, vec![accessible_click(button)])
                .drop_without_applying_deltas();
            assert!(app.menu_open && !app.controls_open && app.player.paused && app.active_status);
            assert!(app.settings_page == page);
            assert_eq!(app.message_until, deadline);
            frame(&ctx, &mut app, 0.2, vec![]).drop_without_applying_deltas();
            assert_eq!(
                rig.builders.borrow().last().unwrap().title.as_deref(),
                Some(page.title())
            );
            let child = rig.output.borrow();
            let output = child.as_ref().unwrap();
            accesskit_node(output, required);
            assert!(
                !output
                    .shapes
                    .iter()
                    .any(|shape| find_text(&shape.shape, absent).is_some())
            );
            assert!(!output.shapes.iter().any(|shape| {
                find_text(&shape.shape, "Welche Animation soll ich zeigen?").is_some()
            }));
        }
    }

    #[test]
    fn wheel_quiet_toggle_is_reversible_and_never_changes_timer_or_animation_pause() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = app(&ctx);
        app.timer.start(crate::timer::Phase::Focus, Instant::now());
        app.player.paused = true;
        for previous in [false, true] {
            app.controls_open = true;
            let output = frame(&ctx, &mut app, 0.0, vec![]);
            let button = accesskit_node(
                &output,
                crate::action_wheel::Action::Quiet.accessible_label(previous),
            )
            .0;
            assert!(
                output
                    .shapes
                    .iter()
                    .any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.starts_with("Fokus · ")) )
            );
            output.drop_without_applying_deltas();
            frame(&ctx, &mut app, 0.1, vec![accessible_click(button)])
                .drop_without_applying_deltas();
            assert_eq!(app.do_not_disturb, !previous);
            assert!(app.timer.has_session() && app.player.paused && !app.controls_open);
        }
    }

    #[test]
    fn wheel_buttons_surround_the_avatar_and_only_their_visible_regions_intercept_clicks() {
        for size in BuddySize::ALL {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = app(&ctx);
            app.size = size;
            app.player.paused = true;
            app.controls_open = true;
            let output = frame(&ctx, &mut app, 0.0, vec![]);
            let avatar = avatar_rect(&output, &app);
            let mut regions = vec![];
            for action in crate::action_wheel::Action::ALL {
                let bounds = accesskit_node(&output, action.accessible_label(false))
                    .1
                    .bounds()
                    .unwrap();
                let rect = Rect::from_min_max(
                    egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
                    egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
                );
                assert!(!rect.expand(8.0).intersects(avatar));
                assert!(rect.width() >= 44.0 && rect.height() >= 44.0);
                assert!(app.hit_map.contains(rect.center()));
                regions.push(rect);
            }
            assert!(regions.iter().any(|r| r.center().x < avatar.left()));
            assert!(regions.iter().any(|r| r.center().x > avatar.right()));
            let center = regions[0].center();
            output.drop_without_applying_deltas();
            app.controls_open = false;
            let output = frame(&ctx, &mut app, 0.1, vec![]);
            assert!(
                !app.hit_map.contains(center),
                "closed menu must release its old native input region"
            );
            assert_controls_hidden(&output);
            output.drop_without_applying_deltas();
        }
    }

    #[test]
    fn wheel_clearance_holds_for_existing_gestures_at_every_buddy_size() {
        for size in BuddySize::ALL {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = app(&ctx);
            app.size = size;
            app.message_until = Instant::now();
            let states = app.engine.animations.keys().cloned().collect::<Vec<_>>();
            for state in states {
                for sample in [0.0, 450.0] {
                    app.player.play(&app.engine, &state, false).unwrap();
                    app.player.paused = false;
                    app.player.advance(&app.engine, sample, 1.0).unwrap();
                    app.player.paused = true;
                    app.controls_open = true;
                    let output = frame(&ctx, &mut app, sample, vec![]);
                    let avatar = avatar_rect(&output, &app);
                    for action in crate::action_wheel::Action::ALL {
                        let b = accesskit_node(&output, action.accessible_label(false))
                            .1
                            .bounds()
                            .unwrap();
                        let rect = Rect::from_min_max(
                            egui::pos2(b.x0 as f32, b.y0 as f32),
                            egui::pos2(b.x1 as f32, b.y1 as f32),
                        );
                        assert!(
                            !rect.expand(8.0).intersects(avatar),
                            "{state}, {size:?}, {sample}"
                        );
                    }
                    output.drop_without_applying_deltas();
                }
            }
        }
    }

    #[test]
    fn constrained_viewport_keeps_actions_accessible_in_a_scrolling_fallback() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = app(&ctx);
        app.size = BuddySize::Small;
        app.controls_open = true;
        let screen = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(220.0, 280.0));
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| app.show_ui(ui),
        );
        output.textures_delta.clear();
        let (button, node) = accesskit_node(&output, "Fokus und Pause öffnen");
        let rect = node.bounds().unwrap();
        assert!(rect.x0 >= 0.0 && rect.y0 >= 0.0 && rect.x1 <= 220.0 && rect.y1 <= 280.0);
        assert!(!app.hit_map.contains(egui::pos2(2.0, 2.0)));
        output.drop_without_applying_deltas();
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                events: vec![accessible_click(button)],
                ..Default::default()
            },
            |ui| app.show_ui(ui),
        );
        output.textures_delta.clear();
        assert!(app.menu_open && app.settings_page == crate::settings_window::Page::Focus);
        assert!(!app.controls_open);
        output.drop_without_applying_deltas();
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
            for label in [
                "Fokus",
                "Erinnerung",
                "Verlauf",
                "Ruhe",
                "Reden",
                "Einstellungen",
                "×",
            ] {
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
        let (avatar, node) = accesskit_node(&output, "Beni: Menü-Rad öffnen");
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
        let (_, node) = accesskit_node(&output, "Beni: Menü-Rad schliessen");
        assert_eq!(node.is_expanded(), Some(true));
        for action in crate::action_wheel::Action::ALL {
            assert_eq!(
                accesskit_node(&output, action.accessible_label(false))
                    .1
                    .role(),
                Role::Button
            );
        }
        output.drop_without_applying_deltas();

        frame(&ctx, &mut app, 0.2, vec![accessible_click(close)]).drop_without_applying_deltas();
        assert!(app.message_until <= Instant::now());
        let output = frame(&ctx, &mut app, 0.3, vec![accessible_click(avatar)]);
        assert!(!app.controls_open);
        assert_eq!(
            accesskit_node(&output, "Beni: Menü-Rad öffnen")
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
        let avatar = accesskit_node(&output, "Beni: Menü-Rad öffnen").0;
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
            "Systemeinstellung beachten",
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
    fn avatar_click_preserves_status_and_restores_the_exact_closed_avatar() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.play("daumen", "Build grün");
        app.player.paused = true;
        app.message_source = Some("codex-test".into());
        app.active_status = true;
        let deadline = app.message_until;
        let output = frame(&ctx, &mut app, 0.0, vec![]);
        assert!(!app.controls_open);
        assert_controls_hidden(&output);
        let avatar = avatar_rect(&output, &app);
        output.drop_without_applying_deltas();

        click(&ctx, &mut app, avatar.center(), 0.1).drop_without_applying_deltas();
        assert!(app.controls_open);
        let output = frame(&ctx, &mut app, 0.2, vec![]);
        for label in [
            "Fokus",
            "Erinnerung",
            "Verlauf",
            "Ruhe",
            "Reden",
            "Einstellungen",
        ] {
            text_rect(&output, label);
        }
        assert!(avatar_rect(&output, &app).width() < avatar.width());
        assert!(
            !output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .iter()
                .any(|c| matches!(
                    c,
                    egui::ViewportCommand::OuterPosition(_) | egui::ViewportCommand::InnerSize(_)
                ))
        );
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
        assert!(app.active_status);
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
        let pause = text_rect(&output, "Einstellungen").center();
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
        scroll_settings(&ctx, &mut app, 0.4).drop_without_applying_deltas();
        let output = frame(&ctx, &mut app, 0.5, vec![]);
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
            for label in [
                "Fokus",
                "Erinnerung",
                "Verlauf",
                "Ruhe",
                "Reden",
                "Einstellungen",
            ] {
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
            let pos = text_rect(&output, "Einstellungen").center();
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
            assert_eq!(app.message, "Hallo von den Tests!");
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
    fn animation_pause_and_quit_remain_reachable_through_tray_and_keyboard() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.handle_tray(Action::Pause, &ctx, None);
        assert!(app.player.paused);
        app.handle_tray(Action::Pause, &ctx, None);
        assert!(!app.player.paused);
        let output = frame(
            &ctx,
            &mut app,
            0.0,
            vec![
                egui::Event::ModifiersChanged(egui::Modifiers {
                    alt: true,
                    ..Default::default()
                }),
                egui::Event::Key {
                    key: egui::Key::Q,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        alt: true,
                        ..Default::default()
                    },
                },
            ],
        );
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
        app.play("hallo", "Hallo!");
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
        let menu = text_rect(&output, "Einstellungen").center();
        output.drop_without_applying_deltas();
        click(&ctx, &mut app, menu, 0.7).drop_without_applying_deltas();
        assert!(app.menu_open);
        app.menu_open = false;
        let output = frame(
            &ctx,
            &mut app,
            0.8,
            vec![
                egui::Event::ModifiersChanged(egui::Modifiers {
                    alt: true,
                    ..Default::default()
                }),
                egui::Event::Key {
                    key: egui::Key::Q,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        alt: true,
                        ..Default::default()
                    },
                },
            ],
        );
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
            Action::Chat,
            Action::CheckUpdates,
            Action::Quit,
        ] {
            app.tray_sender.try_send(action).unwrap();
            let output = ctx.run_logic(&egui::RawInput::default(), |ctx| {
                eframe::App::logic(&mut app, ctx, &mut native_frame);
            });
            match action {
                Action::Show
                | Action::Recover
                | Action::Settings
                | Action::History
                | Action::Chat => {
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
                    } else if action == Action::Chat {
                        assert!(app.chat.open);
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
                Action::CheckUpdates => assert!(app.menu_open),
            }
        }
        app.tray_sender.try_send(Action::Pause).unwrap();
        let _ = ctx.run_logic(&egui::RawInput::default(), |ctx| {
            eframe::App::logic(&mut app, ctx, &mut native_frame);
        });
        assert!(!app.player.paused);
    }

    #[test]
    fn update_action_restores_hidden_buddy_and_explains_missing_updater_without_changing_playback()
    {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.hidden = true;
        app.player.paused = true;
        app.do_not_disturb = true;
        let message = app.message.clone();
        app.handle_tray(Action::CheckUpdates, &ctx, None);
        assert!(app.menu_open && app.updates_info);
        assert!(!app.hidden && app.player.paused && app.do_not_disturb);
        assert_eq!(app.message, message);
        app.open_settings(&ctx);
        assert!(
            !app.updates_info,
            "ordinary settings retain their usual order"
        );
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
            for label in [
                "Fokus",
                "Erinnerung",
                "Verlauf",
                "Ruhe",
                "Reden",
                "Einstellungen",
                "×",
            ] {
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
        app.active_status = true;
        app.message_source = Some("test".into());
        app.player.paused = true;
        let now = Instant::now();
        app.report_error("Testfehler".into(), now);
        app.set_do_not_disturb(true, now);
        assert!(!app.active_status);
        assert!(app.message_until <= now);
        assert!(app.message_source.is_none());
        assert_eq!(app.player.state(), "warten");
        assert!(app.player.paused);
        assert_eq!(app.error.as_ref().unwrap().message, "Testfehler");
        app.play("hallo", "Hallo von mir");
        let deadline = app.message_until;
        app.set_do_not_disturb(true, Instant::now());
        assert_eq!(app.message, "Hallo von mir");
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
            for label in ["Nicht stören ist aktiv", "Fokus", "Einstellungen"] {
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
        assert!(!app.active_status);
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
        app.active_status = true;
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
        assert!(app.player.paused && app.active_status);
        assert_eq!(app.message_until, deadline);
        assert_eq!(app.message_source.as_deref(), Some("build"));
        app.open_history();
        assert!(app.player.paused, "an existing pause stays active");
        app.set_do_not_disturb(true, Instant::now());
        assert_eq!(app.player.state(), "nachdenken");
        assert!(!app.active_status);
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
                accesskit_node(&output, "Beni: Verlauf schliessen")
                    .1
                    .description(),
                Some("Animation: Denken")
            );
            for label in ["Meldungsverlauf (1)", "Leeren"] {
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
        assert!(!app.active_status);
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
            app.player.paused && app.active_status,
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
        assert!(app.active_status && app.player.paused);
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
    fn card_key(key: egui::Key) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }
    }
    fn edit_card(
        ctx: &egui::Context,
        app: &mut BuddyApp,
        rig: &NativeSettings,
        value: &str,
        time: f64,
    ) {
        let mut select = card_key(egui::Key::A);
        if let egui::Event::Key { modifiers, .. } = &mut select {
            modifiers.command = true;
            modifiers.ctrl = true;
        }
        rig.input.borrow_mut().events = vec![
            select,
            card_key(egui::Key::Backspace),
            egui::Event::Text(value.into()),
        ];
        frame(ctx, app, time, vec![]).drop_without_applying_deltas();
        rig.input.borrow_mut().events.clear();
    }
    #[test]
    fn compact_focus_card_focuses_input_preserves_draft_on_escape_and_starts_on_enter() {
        let ctx = egui::Context::default();
        let rig = NativeSettings::new(&ctx);
        let mut app = app(&ctx);
        app.open_page(&ctx, crate::settings_window::Page::Focus);
        frame(&ctx, &mut app, 0.0, vec![]).drop_without_applying_deltas();
        let builder = rig.builders.borrow().last().unwrap().clone();
        assert_eq!(builder.inner_size, Some(egui::vec2(348.0, 332.0)));
        assert_eq!(builder.decorations, Some(false));
        assert_eq!(builder.transparent, Some(true));
        {
            let child = rig.output.borrow();
            let child = child.as_ref().unwrap();
            let (input, node) = accesskit_node(child, "Dauer in Minuten");
            assert_eq!(node.value(), Some("25"));
            assert_eq!(
                child
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .focus,
                input
            );
            for name in [
                "Dauer in Minuten",
                "Fokus starten",
                "Eingabekarte schliessen",
            ] {
                let bounds = accesskit_node(child, name).1.bounds().unwrap();
                assert!(
                    bounds.x0 >= 0.0
                        && bounds.y0 >= 0.0
                        && bounds.x1 <= 348.0
                        && bounds.y1 <= 332.0,
                    "{name}: {bounds:?}"
                );
            }
        }
        edit_card(&ctx, &mut app, &rig, "17", 0.1);
        rig.input.borrow_mut().events = vec![card_key(egui::Key::Escape)];
        frame(&ctx, &mut app, 0.2, vec![]).drop_without_applying_deltas();
        assert!(!app.menu_open && !app.timer.has_session());
        rig.input.borrow_mut().events.clear();
        app.open_page(&ctx, crate::settings_window::Page::Focus);
        frame(&ctx, &mut app, 0.3, vec![]).drop_without_applying_deltas();
        assert_eq!(
            accesskit_node(rig.output.borrow().as_ref().unwrap(), "Dauer in Minuten")
                .1
                .value(),
            Some("17")
        );
        rig.input.borrow_mut().events = vec![card_key(egui::Key::Enter)];
        frame(&ctx, &mut app, 0.4, vec![]).drop_without_applying_deltas();
        assert!(!app.menu_open && app.timer.has_session());
        let mut storage = TestStorage::default();
        app.timer.save(&mut storage);
        let minutes: [u16; 2] = eframe::get_value(&storage, "timer-minutes").unwrap();
        assert_eq!(minutes, [17, 5]);
        rig.input.borrow_mut().events.clear();
        app.open_settings(&ctx);
        frame(&ctx, &mut app, 0.5, vec![]).drop_without_applying_deltas();
        let builder = rig.builders.borrow().last().unwrap().clone();
        assert_eq!(builder.inner_size, Some(egui::vec2(520.0, 680.0)));
        assert_eq!(builder.decorations, Some(true));
    }
    #[test]
    fn invalid_card_minutes_never_start_a_timer_or_close_the_card() {
        for invalid in ["", "0", "181", "abc"] {
            let ctx = egui::Context::default();
            let rig = NativeSettings::new(&ctx);
            let mut app = app(&ctx);
            app.open_page(&ctx, crate::settings_window::Page::Focus);
            frame(&ctx, &mut app, 0.0, vec![]).drop_without_applying_deltas();
            edit_card(&ctx, &mut app, &rig, invalid, 0.1);
            rig.input.borrow_mut().events = vec![card_key(egui::Key::Enter)];
            frame(&ctx, &mut app, 0.2, vec![]).drop_without_applying_deltas();
            assert!(app.menu_open && !app.timer.has_session(), "{invalid}");
        }
    }
    #[test]
    fn reminder_card_preserves_cancelled_text_and_schedules_exactly_once_on_enter() {
        let ctx = egui::Context::default();
        let rig = NativeSettings::new(&ctx);
        let mut app = app(&ctx);
        app.open_page(&ctx, crate::settings_window::Page::Reminders);
        frame(&ctx, &mut app, 0.0, vec![]).drop_without_applying_deltas();
        {
            let child = rig.output.borrow();
            let child = child.as_ref().unwrap();
            assert_eq!(
                child
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .focus,
                accesskit_node(child, "Erinnerungstext").0
            );
        }
        edit_card(&ctx, &mut app, &rig, "Wasser trinken", 0.1);
        rig.input.borrow_mut().events = vec![card_key(egui::Key::Escape)];
        frame(&ctx, &mut app, 0.2, vec![]).drop_without_applying_deltas();
        assert!(!app.menu_open && app.reminders.count() == 0);
        rig.input.borrow_mut().events.clear();
        app.open_page(&ctx, crate::settings_window::Page::Reminders);
        frame(&ctx, &mut app, 0.3, vec![]).drop_without_applying_deltas();
        assert_eq!(
            accesskit_node(rig.output.borrow().as_ref().unwrap(), "Erinnerungstext")
                .1
                .value(),
            Some("Wasser trinken")
        );
        rig.input.borrow_mut().events = vec![card_key(egui::Key::Enter)];
        frame(&ctx, &mut app, 0.4, vec![]).drop_without_applying_deltas();
        assert!(!app.menu_open && app.reminders.count() == 1);
        rig.input.borrow_mut().events.clear();
        app.open_page(&ctx, crate::settings_window::Page::Reminders);
        frame(&ctx, &mut app, 0.5, vec![]).drop_without_applying_deltas();
        rig.input.borrow_mut().events = vec![card_key(egui::Key::Enter)];
        frame(&ctx, &mut app, 0.6, vec![]).drop_without_applying_deltas();
        assert!(app.menu_open && app.reminders.count() == 1);
        rig.input.borrow_mut().events.clear();
        edit_card(&ctx, &mut app, &rig, "Weiterer Entwurf", 0.7);
        for index in 0..9 {
            app.reminders
                .schedule(&format!("Erinnerung {index}"), 10, Instant::now())
                .unwrap();
        }
        rig.input.borrow_mut().events = vec![card_key(egui::Key::Enter)];
        frame(&ctx, &mut app, 0.8, vec![]).drop_without_applying_deltas();
        assert!(app.menu_open && app.reminders.count() == 10);
        app.handle_tray(Action::Hide, &ctx, None);
        assert!(!app.menu_open && app.hidden && app.reminders.count() == 10);
    }
}
