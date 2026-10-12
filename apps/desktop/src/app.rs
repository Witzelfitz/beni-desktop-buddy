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
impl eframe::App for BuddyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
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
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "buddy-settings", &self.settings);
    }
}
