use buddy_core::{ATLAS_PNG, Engine, Player, Settings};
use eframe::egui::{self, Color32, FontId, Rect, Sense, TextureHandle, Vec2};
use std::time::{Duration, Instant};

pub struct BuddyApp {
    engine: Engine,
    player: Player,
    texture: TextureHandle,
    settings: Settings,
    last_tick: Instant,
    message: String,
    message_until: Instant,
    menu_open: bool,
    speech_draft: String,
    error: Option<String>,
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
        Ok(Self {
            engine,
            player,
            texture,
            settings,
            last_tick: Instant::now(),
            message: "Hoi! Ich bin Beni. Zieh mich an deinen Lieblingsplatz.".into(),
            message_until: Instant::now() + Duration::from_secs(10),
            menu_open: false,
            speech_draft: "Schön bist du da!".into(),
            error: None,
        })
    }
    fn play(&mut self, state: &str, message: &str) {
        if let Err(e) = self.player.play(&self.engine, state, true) {
            self.error = Some(e.to_string());
            return;
        }
        self.message = message.chars().take(400).collect();
        self.message_until = Instant::now() + Duration::from_secs(10);
    }
    fn paint_avatar(&mut self, ui: &mut egui::Ui, stage: Rect) {
        let pose = match self.player.pose(&self.engine, &self.settings) {
            Ok(pose) => pose,
            Err(e) => {
                self.error = Some(e.to_string());
                return;
            }
        };
        let corners = match self
            .engine
            .corners(self.player.state(), &pose, self.settings.mirrored)
        {
            Ok(corners) => corners,
            Err(e) => {
                self.error = Some(e.to_string());
                return;
            }
        };
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
        ui.painter().add(egui::Shape::mesh(mesh));
        let response = ui.interact(stage, ui.id().with("avatar"), Sense::click_and_drag());
        if response.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if response.clicked() {
            self.play("hallo", "Hoi! Ich bin dein Desktop-Buddy.");
        }
        response.on_hover_text("Ziehen: verschieben · Klicken: Hallo sagen");
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
                    .max_height(110.0)
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
        if let Err(e) = self
            .player
            .advance(&self.engine, delta.min(1000.0), self.settings.speed)
        {
            self.error = Some(e.to_string());
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
            let speech = Rect::from_min_size(
                ui.cursor().min + egui::vec2(12.0, 0.0),
                egui::vec2(ui.available_width() - 24.0, 72.0),
            );
            if now < self.message_until || self.error.is_some() {
                ui.painter()
                    .rect_filled(speech, 16, Color32::from_rgb(246, 249, 237));
                let text = self.error.as_deref().unwrap_or(&self.message);
                let galley = ui.painter().layout(
                    text.to_string(),
                    FontId::proportional(14.0),
                    Color32::from_rgb(28, 41, 35),
                    speech.width() - 28.0,
                );
                ui.painter().with_clip_rect(speech.shrink(8.0)).galley(
                    speech.min + egui::vec2(14.0, 12.0),
                    galley,
                    Color32::BLACK,
                );
            }
            ui.allocate_space(Vec2::new(ui.available_width(), 78.0));
            let (_, stage) = ui.allocate_space(Vec2::new(ui.available_width(), 295.0));
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
impl eframe::App for BuddyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show_ui(ui);
    }
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "buddy-settings", &self.settings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app(ctx: &egui::Context) -> BuddyApp {
        let icon = eframe::icon_data::from_png_bytes(ATLAS_PNG).unwrap();
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [icon.width as usize, icon.height as usize],
            &icon.rgba,
        );
        BuddyApp {
            engine: Engine::embedded().unwrap(),
            player: Player::default(),
            texture: ctx.load_texture("test-atlas", image, egui::TextureOptions::LINEAR),
            settings: Settings::default(),
            last_tick: Instant::now(),
            message: "Hoi!".into(),
            message_until: Instant::now() + Duration::from_secs(10),
            menu_open: false,
            speech_draft: "Hoi von den Tests!".into(),
            error: None,
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
}
