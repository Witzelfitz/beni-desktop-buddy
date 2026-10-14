mod ollama;
mod presentation;
pub use presentation::Cue;
use presentation::Presentation;
pub const EXTRA_HEIGHT: f32 = 340.0;
#[cfg(test)]
pub fn dimensions(base: egui::Vec2, open: bool, monitor: Option<egui::Vec2>) -> egui::Vec2 {
    dimensions_with_space(base, open, monitor, None)
}
pub fn dimensions_with_space(
    base: egui::Vec2,
    open: bool,
    monitor: Option<egui::Vec2>,
    space_above: Option<f32>,
) -> egui::Vec2 {
    if !open {
        return base;
    }
    let monitor = monitor.filter(|m| m.x.is_finite() && m.y.is_finite() && m.x > 0.0 && m.y > 0.0);
    if monitor.is_some_and(|m| {
        (m.y - 48.0 < base.y + 200.0
            || space_above.is_some_and(|space| space < (EXTRA_HEIGHT.min(m.y - 48.0 - base.y))))
            && m.x >= base.x + 440.0
    }) {
        return egui::vec2(base.x + 440.0, base.y);
    }
    let height = monitor.map_or(base.y + EXTRA_HEIGHT, |m| {
        (base.y + EXTRA_HEIGHT).min(m.y - 48.0)
    });
    egui::vec2(base.x.max(420.0), height)
}
pub fn beside(base: egui::Vec2, actual: egui::Vec2) -> bool {
    actual.x >= base.x + 430.0 && actual.y < base.y + 150.0
}
pub fn offset(base: egui::Vec2, actual: egui::Vec2) -> egui::Vec2 {
    egui::vec2(
        (actual.x - base.x) * if beside(base, actual) { 1.0 } else { 0.5 },
        actual.y - base.y,
    )
}
use buddy_chat::{
    CancellationToken, ChatError, ChatProvider, ChatSession, Phase, RequestId, StreamEvent,
};
use eframe::egui;
use ollama::{Model, Ollama};
use std::{
    sync::mpsc::{Receiver, SyncSender, TryRecvError, TrySendError, sync_channel},
    thread::JoinHandle,
    time::{Duration, Instant},
};

enum Event {
    Models(Result<Vec<Model>, ChatError>),
    Reply(RequestId, StreamEvent),
}
struct Worker {
    receive: Receiver<Event>,
    cancellation: CancellationToken,
    _handle: JoinHandle<()>,
    request_id: Option<RequestId>,
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

pub struct Chat {
    pub session: ChatSession,
    pub open: bool,
    enabled: bool,
    selected: String,
    models: Vec<Model>,
    discovered: bool,
    connection_error: Option<ChatError>,
    draft: String,
    pending_input: String,
    worker: Option<Worker>,
    provider: Ollama,
    clock: Instant,
    presentation: Option<Presentation>,
    connection_open: bool,
    focused: bool,
}
impl Chat {
    pub fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        let selected: String = storage
            .and_then(|s| eframe::get_value(s, "chat-ollama-model"))
            .unwrap_or_default();
        Self {
            session: ChatSession::default(),
            open: false,
            enabled: storage
                .and_then(|s| eframe::get_value(s, "chat-ollama-enabled"))
                .unwrap_or(false),
            selected: if buddy_chat::valid_model_name(&selected) {
                selected
            } else {
                String::new()
            },
            models: Vec::new(),
            discovered: false,
            connection_error: None,
            draft: String::new(),
            pending_input: String::new(),
            worker: None,
            provider: Ollama::default(),
            clock: Instant::now(),
            presentation: None,
            connection_open: false,
            focused: false,
        }
    }
    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "chat-ollama-enabled", &self.enabled);
        eframe::set_value(storage, "chat-ollama-model", &self.selected);
    }
    fn now_ms(&self) -> u64 {
        self.clock.elapsed().as_millis().min(u64::MAX as u128) as u64
    }
    pub fn open(&mut self, ctx: &egui::Context) {
        self.open = true;
        if self.enabled && !self.discovered && self.worker.is_none() {
            self.refresh(ctx);
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        ctx.request_repaint();
    }
    pub fn close(&mut self) {
        self.cancel();
        self.open = false;
        self.focused = false;
    }
    pub fn escape(&mut self, ctx: &egui::Context) {
        if self.focused {
            if let Some(id) = ctx.memory(|m| m.focused()) {
                ctx.memory_mut(|m| m.surrender_focus(id));
            }
            self.focused = false;
        } else {
            self.close();
        }
    }
    pub fn cancel(&mut self) {
        if let Some(worker) = &self.worker {
            worker.cancellation.cancel();
        }
        self.session.cancel();
        self.presentation = None;
    }
    pub fn busy(&self) -> bool {
        self.session.busy() || self.presentation.as_ref().is_some_and(Presentation::active)
    }
    pub(crate) fn receive(&mut self, id: RequestId, event: StreamEvent, now: u64) {
        let finished = matches!(&event, StreamEvent::Finished { .. });
        if self.session.receive(id, event, now)
            && finished
            && self.open
            && matches!(self.session.phase(), Phase::Completed { .. })
            && let Some(reply) = self.session.reply()
        {
            self.presentation = Some(Presentation::new(id, reply.clone(), now));
        }
    }
    pub(crate) fn next_part(&mut self) {
        let now = self.now_ms();
        if let Some(p) = &mut self.presentation {
            p.next(now);
        }
    }
    pub fn poll(&mut self) {
        let now = self.now_ms();
        self.session.advance(now);
        if let Some(presentation) = &mut self.presentation {
            presentation.advance(now);
        }
        if let Some(worker) = &self.worker {
            let mut disconnected = false;
            let mut replies = Vec::new();
            for _ in 0..32 {
                let event = match worker.receive.try_recv() {
                    Ok(event) => event,
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                };
                match event {
                    Event::Models(result) if !worker.cancellation.is_cancelled() => {
                        self.discovered = true;
                        match result {
                            Ok(models) => {
                                self.models = models;
                                self.connection_error = None;
                            }
                            Err(error) => {
                                self.models.clear();
                                self.connection_error = Some(error);
                            }
                        }
                        if !self.models.iter().any(|m| m.name == self.selected) {
                            self.selected.clear();
                        }
                    }
                    Event::Reply(id, event) => {
                        replies.push((id, event));
                    }
                    _ => {}
                }
            }
            let request_id = worker.request_id;
            for (id, event) in replies {
                self.receive(id, event, now);
            }
            if disconnected {
                // Disconnection proves all buffered events were consumed.
                // Dropping a handle never blocks the GUI to join a worker.
                if let Some(id) = request_id {
                    self.session
                        .receive(id, StreamEvent::Failed(ChatError::Protocol), now);
                }
                self.worker.take();
            }
        }
    }
    fn refresh(&mut self, ctx: &egui::Context) {
        if self.worker.is_some() {
            return;
        }
        self.enabled = true;
        self.presentation = None;
        self.session.clear();
        self.pending_input.clear();
        self.models.clear();
        self.discovered = false;
        self.connection_error = None;
        let provider = self.provider.clone();
        let (send, receive) = sync_channel(16);
        let token = CancellationToken::default();
        let cancellation = token.clone();
        let wake = ctx.clone();
        let handle = std::thread::spawn(move || {
            let result = provider.discover(&token);
            let _ = send.try_send(Event::Models(result));
            wake.request_repaint_of(egui::ViewportId::ROOT);
        });
        self.worker = Some(Worker {
            receive,
            cancellation,
            _handle: handle,
            request_id: None,
        });
    }
    fn send(&mut self, ctx: &egui::Context) {
        if self.worker.is_some() || !self.enabled {
            return;
        }
        let Some(model) = self
            .models
            .iter()
            .find(|m| m.name == self.selected)
            .cloned()
        else {
            return;
        };
        let (request, token) = match self.session.start(&model.name, &self.draft, self.now_ms()) {
            Ok(value) => value,
            Err(error) => {
                self.connection_error = Some(error);
                return;
            }
        };
        self.presentation = None;
        self.connection_open = false;
        self.pending_input = self.draft.trim().into();
        self.draft.clear();
        self.connection_error = None;
        let id = request.id();
        let provider = self.provider.with_model(model);
        let (send, receive) = sync_channel(16);
        let cancellation = token.clone();
        let wake = ctx.clone();
        let handle = std::thread::spawn(move || {
            let started = Instant::now();
            let mut emit =
                |event| send_event(&send, Event::Reply(id, event), &token, started, &wake);
            if let Err(error) = provider.stream(&request, &token, &mut emit) {
                let _ = emit(StreamEvent::Failed(error));
            }
            wake.request_repaint_of(egui::ViewportId::ROOT);
        });
        self.worker = Some(Worker {
            receive,
            cancellation,
            _handle: handle,
            request_id: Some(id),
        });
        ctx.request_repaint_of(egui::ViewportId::ROOT);
    }
    fn disconnect(&mut self) {
        self.cancel();
        self.enabled = false;
        self.selected.clear();
        self.models.clear();
        self.discovered = false;
        self.connection_error = None;
        self.presentation = None;
        self.session.clear();
        self.pending_input.clear();
    }
    pub fn cue(&self) -> Option<Cue> {
        if !self.open {
            return None;
        }
        match self.session.phase() {
            Phase::Waiting => Some(Cue::Thinking),
            Phase::Speaking => Some(Cue::Speaking),
            Phase::Completed { .. } => self.presentation.as_ref().and_then(Presentation::cue),
            _ => None,
        }
    }
    pub fn show(&mut self, ui: &mut egui::Ui, beside: bool) -> egui::Rect {
        let fill = egui::Color32::from_rgb(246, 249, 237);
        let rect = egui::Frame::new()
            .fill(fill)
            .corner_radius(18)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.set_width(ui.available_width().max(200.0));
                *ui.visuals_mut() = egui::Visuals::light();
                ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(28, 41, 35));
                ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::from_rgb(220, 231, 205);
                ui.visuals_mut().widgets.hovered.bg_fill = egui::Color32::from_rgb(200, 220, 185);
                self.contents(ui);
            })
            .response
            .rect;
        // The tail stays in this transparent buddy surface and moves with Beni.
        let points = if beside {
            let center = rect.right_center();
            vec![
                center + egui::vec2(-1.0, -10.0),
                center + egui::vec2(-1.0, 10.0),
                center + egui::vec2(12.0, 0.0),
            ]
        } else {
            let center = rect.center_bottom();
            vec![
                center + egui::vec2(-10.0, -1.0),
                center + egui::vec2(10.0, -1.0),
                center + egui::vec2(0.0, 12.0),
            ]
        };
        ui.painter().add(egui::Shape::convex_polygon(
            points,
            fill,
            egui::Stroke::NONE,
        ));
        if self.open {
            self.focused = ui.ctx().memory(|m| m.focused()).is_some();
        }
        rect.expand2(if beside {
            egui::vec2(12.0, 0.0)
        } else {
            egui::vec2(0.0, 12.0)
        })
    }
    fn connection(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        if !self.enabled {
            ui.label("Verbinde ein vorhandenes lokales Ollama-Modell. Nur dieses Gespräch und der angezeigte Kontext werden an den lokalen Dienst gesendet.");
            if ui.button("Lokales Ollama verbinden").clicked() {
                self.refresh(&ctx);
            }
            return;
        }
        let previous = self.selected.clone();
        ui.add_enabled_ui(self.worker.is_none(), |ui| {
            egui::ComboBox::from_id_salt("chat-model")
                .width(ui.available_width().min(330.0))
                .selected_text(if self.selected.is_empty() {
                    "Modell wählen"
                } else {
                    &self.selected
                })
                .show_ui(ui, |ui| {
                    for model in &self.models {
                        if ui
                            .selectable_value(
                                &mut self.selected,
                                model.name.clone(),
                                format!(
                                    "{} · {:.1} GiB",
                                    model.name,
                                    model.size as f64 / 1073741824.0
                                ),
                            )
                            .clicked()
                        {
                            ui.close();
                        }
                    }
                });
            if ui.button("Modelle aktualisieren").clicked() {
                self.refresh(&ctx);
            }
        });
        if self.selected != previous {
            self.session.clear();
            self.presentation = None;
            self.pending_input.clear();
            self.connection_open = false;
        }
        if !self.discovered && self.worker.is_some() {
            ui.label("Modelle prüfen …");
        } else if self.discovered && self.models.is_empty() && self.connection_error.is_none() {
            ui.label("Kein geeignetes lokales Textmodell gefunden.");
        }
        if let Some(error) = self.connection_error {
            ui.colored_label(egui::Color32::DARK_RED, error.to_string());
        }
        ui.small("Gespräch nur in dieser Sitzung. Statusmeldungen bleiben getrennt.");
        if !self.draft.trim().is_empty() {
            egui::CollapsingHeader::new("Kontext für diese Anfrage").show(ui, |ui| {
                match self.session.preview(&self.draft) {
                    Ok(messages) => {
                        for message in messages {
                            ui.strong(match message.role() {
                                buddy_chat::Role::System => "Benis Rolle",
                                buddy_chat::Role::User => "Du",
                                buddy_chat::Role::Assistant => "Beni",
                            });
                            ui.add(egui::Label::new(message.text()).wrap());
                        }
                    }
                    Err(error) => {
                        ui.label(error.to_string());
                    }
                }
            });
        }
        if ui.button("Ollama trennen").clicked() {
            self.disconnect();
        }
    }
    fn contents(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.strong("Beni");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let close = ui.button("×").on_hover_text("Gespräch schliessen");
                close.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Gespräch schliessen")
                });
                if close.clicked() {
                    self.close();
                }
            });
        });
        let ready = self.enabled
            && self.worker.is_none()
            && self.models.iter().any(|m| m.name == self.selected);
        let body_height = (ui.available_height() - 112.0).max(20.0);
        egui::ScrollArea::vertical()
            .id_salt("beni-conversation-bubble")
            .max_height(body_height)
            .min_scrolled_height(body_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let required =
                    !self.enabled || self.selected.is_empty() || self.connection_error.is_some();
                let header = egui::CollapsingHeader::new("Verbindung & Kontext")
                    .open(Some(required || self.connection_open))
                    .show(ui, |ui| self.connection(ui));
                if header.header_response.clicked() && !required {
                    self.connection_open = !self.connection_open;
                }
                if self.worker.is_some() && !self.session.busy() {
                    ui.small("Modelle prüfen …");
                }
                if let Some(presentation) = &self.presentation {
                    ui.add(
                        egui::Label::new(egui::RichText::new(presentation.text()).size(15.0))
                            .wrap(),
                    );
                    if presentation.parts() > 1 {
                        ui.small(format!(
                            "Abschnitt {} von {}",
                            presentation.index() + 1,
                            presentation.parts()
                        ));
                    }
                } else if self.session.busy() {
                    ui.label("Ich denke über deine Nachricht nach …");
                } else if self.session.response().is_empty()
                    || !matches!(self.session.phase(), Phase::Completed { .. })
                {
                    ui.label("Hallo! Worüber möchtest du sprechen?");
                } else {
                    ui.add(egui::Label::new(self.session.response()).wrap());
                }
                match self.session.phase() {
                    Phase::Failed(error) => {
                        ui.colored_label(egui::Color32::DARK_RED, error.to_string());
                    }
                    Phase::Cancelled => {
                        ui.label("Anfrage abgebrochen.");
                    }
                    _ => {}
                }
                if self.session.history_turns() > 0 {
                    egui::CollapsingHeader::new("Gespräch nachlesen").show(ui, |ui| {
                        for (input, output) in self.session.history() {
                            ui.strong("Du");
                            ui.add(egui::Label::new(input).wrap());
                            ui.strong("Beni");
                            ui.add(egui::Label::new(output).wrap());
                            ui.add_space(8.0);
                        }
                    });
                }
            });
        let edit = ui.add_enabled(
            ready,
            egui::TextEdit::multiline(&mut self.draft)
                .desired_rows(2)
                .desired_width(f32::INFINITY)
                .char_limit(buddy_chat::MAX_INPUT_BYTES)
                .hint_text("Schreibe Beni eine Nachricht …"),
        );
        edit.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::TextEdit,
                edit.enabled(),
                "Nachricht an Beni",
            )
        });
        let shortcut = ready
            && edit.has_focus()
            && ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter));
        edit.on_hover_text("Cmd+Enter auf macOS, Ctrl+Enter auf Windows/Linux sendet. Enter fügt eine neue Zeile ein.");
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    ready && !self.draft.trim().is_empty(),
                    egui::Button::new("Senden"),
                )
                .clicked()
                || shortcut
            {
                self.send(ui.ctx());
            }
            if ui
                .add_enabled(self.worker.is_some(), egui::Button::new("Abbrechen"))
                .clicked()
            {
                self.cancel();
            }
            let next = self
                .presentation
                .as_ref()
                .is_some_and(Presentation::has_next);
            if ui.add_enabled(next, egui::Button::new("Weiter")).clicked() {
                self.next_part();
            }
            if ui
                .add_enabled(self.worker.is_none(), egui::Button::new("Gespräch leeren"))
                .clicked()
            {
                self.session.clear();
                self.presentation = None;
                self.pending_input.clear();
            }
        });
        if self.worker.is_some() || self.presentation.as_ref().is_some_and(Presentation::active) {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
    }
}
fn send_event(
    send: &SyncSender<Event>,
    mut event: Event,
    cancellation: &CancellationToken,
    started: Instant,
    ctx: &egui::Context,
) -> Result<(), ChatError> {
    loop {
        if cancellation.is_cancelled() {
            return Err(ChatError::Cancelled);
        }
        if started.elapsed() >= Duration::from_millis(buddy_chat::REQUEST_TIMEOUT_MS) {
            return Err(ChatError::Timeout);
        }
        match send.try_send(event) {
            Ok(()) => {
                ctx.request_repaint_of(egui::ViewportId::ROOT);

                return Ok(());
            }
            Err(TrySendError::Full(returned)) => {
                event = returned;
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(TrySendError::Disconnected(_)) => return Err(ChatError::Cancelled),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Storage(std::collections::HashMap<String, String>);
    impl eframe::Storage for Storage {
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
    fn escape_finishes_editing_even_if_egui_already_cleared_focus_then_cancels() {
        let ctx = egui::Context::default();
        let mut chat = Chat::load(None);
        chat.open = true;
        chat.focused = true;
        let (_, token) = chat.session.start("test", "Frage", 0).unwrap();
        assert!(ctx.memory(|m| m.focused()).is_none());
        chat.escape(&ctx);
        assert!(chat.open && chat.session.busy() && !token.is_cancelled());
        chat.escape(&ctx);
        assert!(!chat.open && !chat.session.busy() && token.is_cancelled());
    }

    #[test]
    fn opt_in_and_model_persist_but_conversations_never_do() {
        let mut chat = Chat::load(None);
        assert!(!chat.enabled && chat.worker.is_none());
        chat.enabled = true;
        chat.selected = "local:small".into();
        let (request, _) = chat
            .session
            .start("local:small", "private-question", 0)
            .unwrap();
        chat.session
            .receive(request.id(), StreamEvent::Delta("private-answer".into()), 1);
        chat.session.receive(
            request.id(),
            StreamEvent::Finished {
                reason: buddy_chat::FinishReason::Stop,
                usage: buddy_chat::Usage::default(),
            },
            2,
        );
        let mut storage = Storage::default();
        chat.save(&mut storage);
        assert_eq!(storage.0.len(), 2);
        assert!(
            storage
                .0
                .values()
                .all(|v| !v.contains("private-question") && !v.contains("private-answer"))
        );
        let mut loaded = Chat::load(Some(&storage));
        assert!(loaded.enabled);
        assert_eq!(loaded.selected, "local:small");
        assert_eq!(loaded.session.history_turns(), 0);
        assert!(!loaded.open && loaded.worker.is_none());
        loaded.disconnect();
        loaded.save(&mut storage);
        assert!(!Chat::load(Some(&storage)).enabled);
        eframe::set_value(&mut storage, "chat-ollama-model", &"bad\nname");
        assert!(Chat::load(Some(&storage)).selected.is_empty());
    }
    #[test]
    fn bounded_channel_wait_is_cancellable_and_receiver_drop_is_safe() {
        let ctx = egui::Context::default();
        let (send, receive) = sync_channel(1);
        send.try_send(Event::Models(Ok(Vec::new())))
            .unwrap_or_else(|_| panic!("empty channel"));
        let token = CancellationToken::default();
        let cancel = token.clone();
        let worker = std::thread::spawn(move || {
            send_event(
                &send,
                Event::Models(Ok(Vec::new())),
                &token,
                Instant::now(),
                &ctx,
            )
        });
        cancel.cancel();
        assert_eq!(worker.join().unwrap(), Err(ChatError::Cancelled));
        drop(receive);
        let (send, receive) = sync_channel(1);
        drop(receive);
        assert_eq!(
            send_event(
                &send,
                Event::Models(Ok(Vec::new())),
                &CancellationToken::default(),
                Instant::now(),
                &egui::Context::default()
            ),
            Err(ChatError::Cancelled)
        );
    }
    #[test]
    fn worker_completion_drains_terminal_event_before_reporting_eof() {
        let mut chat = Chat::load(None);
        let (request, token) = chat.session.start("test", "question", 0).unwrap();
        let id = request.id();
        let (send, receive) = sync_channel(16);
        send.send(Event::Reply(id, StreamEvent::Delta("complete".into())))
            .unwrap_or_else(|_| panic!("live receiver"));
        send.send(Event::Reply(
            id,
            StreamEvent::Finished {
                reason: buddy_chat::FinishReason::Stop,
                usage: buddy_chat::Usage::default(),
            },
        ))
        .unwrap_or_else(|_| panic!("live receiver"));
        drop(send);
        chat.worker = Some(Worker {
            receive,
            cancellation: token,
            _handle: std::thread::spawn(|| {}),
            request_id: Some(id),
        });
        chat.poll();
        assert!(matches!(chat.session.phase(), Phase::Completed { .. }));
        assert_eq!(chat.session.response(), "complete");
        assert!(chat.worker.is_none());
        let (request, token) = chat
            .session
            .start("test", "incomplete", chat.now_ms())
            .unwrap();
        let id = request.id();
        let (send, receive) = sync_channel(16);
        drop(send);
        chat.worker = Some(Worker {
            receive,
            cancellation: token,
            _handle: std::thread::spawn(|| {}),
            request_id: Some(id),
        });
        chat.poll();
        assert_eq!(chat.session.phase(), Phase::Failed(ChatError::Protocol));
    }
}
