mod ollama;
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
        ctx.send_viewport_cmd_to(id(), egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd_to(id(), egui::ViewportCommand::Focus);
        ctx.request_repaint();
    }
    pub fn close(&mut self) {
        self.cancel();
        self.open = false;
    }
    pub fn cancel(&mut self) {
        if let Some(worker) = &self.worker {
            worker.cancellation.cancel();
        }
        self.session.cancel();
    }
    pub fn busy(&self) -> bool {
        self.session.busy()
    }
    pub fn poll(&mut self) {
        let now = self.now_ms();
        self.session.advance(now);
        if let Some(worker) = &self.worker {
            let mut disconnected = false;
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
                        self.session.receive(id, event, now);
                    }
                    _ => {}
                }
            }
            if disconnected {
                // Disconnection proves all buffered events were consumed.
                // Dropping a handle never blocks the GUI to join a worker.
                if let Some(id) = worker.request_id {
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
            wake.request_repaint_of(id());
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
            wake.request_repaint_of(crate::chat::id());
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
        self.session.clear();
        self.pending_input.clear();
    }
    pub fn show(&mut self, ctx: &egui::Context) -> Option<egui::Rect> {
        if !self.open {
            return None;
        }
        if ctx.embed_viewports() {
            let mut open = true;
            let response = egui::Window::new("Mit Beni reden")
                .open(&mut open)
                .default_size([640.0, 720.0])
                .min_size([420.0, 520.0])
                .resizable(true)
                .collapsible(false)
                .show(ctx, |ui| self.contents(ui));
            if !open {
                self.close();
            }
            return response.map(|r| r.response.rect);
        }
        ctx.show_viewport_immediate(id(), builder(), |ui, _| {
            let child = ui.ctx().clone();
            let escape = child.input(|i| i.key_pressed(egui::Key::Escape));
            if child.input(|i| i.viewport().close_requested()) || (escape && !self.focused) {
                self.close();
                child.request_repaint_of(egui::ViewportId::ROOT);
                return;
            }
            if escape && let Some(focused) = child.memory(|m| m.focused()) {
                child.memory_mut(|m| m.surrender_focus(focused));
            }
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().window_fill());
            egui::Frame::new().inner_margin(16).show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                self.contents(ui);
            });
            self.focused = child.memory(|m| m.focused()).is_some();
            if self.worker.is_some() {
                child.request_repaint_after(Duration::from_millis(100));
            }
        });
        None
    }
    fn contents(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.heading("Beni & du");
        if !self.enabled {
            ui.label("Verbinde Beni mit deinem lokalen Ollama und wähle ein bereits installiertes Chatmodell.");
            ui.label("Dein eingegebener Text und der angezeigte Chatkontext werden an den lokalen Dienst gesendet. Statusmeldungen bleiben getrennt.");
            if ui.button("Lokales Ollama verbinden").clicked() {
                self.refresh(&ctx);
            }
            if ui.button("Später").clicked() {
                self.close();
            }
            return;
        }
        ui.horizontal_wrapped(|ui| {
            ui.label("Ollama lokal");
            let previous = self.selected.clone();
            ui.add_enabled_ui(self.worker.is_none(), |ui| {
                egui::ComboBox::from_id_salt("chat-model")
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
                self.pending_input.clear();
            }
        });
        ui.small("Nur dieses Gespräch geht an deinen lokalen Ollama-Dienst. Keine Statusmeldungen, Dateien oder Bildschirmdaten.");
        if !self.discovered && self.worker.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Modelle prüfen …");
            });
        } else if self.discovered && self.models.is_empty() && self.connection_error.is_none() {
            ui.label("Kein lokales Chatmodell gefunden. Bild-, Embedding- und Cloud-Modelle werden nicht angeboten.");
        }
        if let Some(error) = self.connection_error {
            ui.colored_label(egui::Color32::LIGHT_RED, error.to_string());
        }
        ui.separator();
        let height = (ui.available_height() - 160.0).max(70.0);
        egui::ScrollArea::vertical()
            .id_salt("beni-chat-history")
            .max_height(height)
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
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
                if self.session.history_turns() == 0 && self.pending_input.is_empty() {
                    ui.label("Hoi! Worüber möchtest du reden?");
                }
                for (input, output) in self.session.history() {
                    ui.strong("Du");
                    ui.add(egui::Label::new(input).wrap());
                    ui.strong("Beni");
                    ui.add(egui::Label::new(output).wrap());
                    ui.add_space(12.0);
                }
                if !self.pending_input.is_empty()
                    && !matches!(self.session.phase(), Phase::Completed { .. })
                {
                    ui.strong("Du");
                    ui.add(egui::Label::new(&self.pending_input).wrap());
                    ui.strong("Beni");
                    ui.add(egui::Label::new(self.session.response()).wrap());
                }
            });
        match self.session.phase() {
            Phase::Waiting => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Beni denkt …");
                });
            }
            Phase::Speaking => {
                ui.label("Beni antwortet …");
            }
            Phase::Failed(error) => {
                ui.colored_label(egui::Color32::LIGHT_RED, error.to_string());
            }
            Phase::Cancelled => {
                ui.label("Anfrage abgebrochen.");
            }
            Phase::Completed {
                reason: buddy_chat::FinishReason::Length,
                ..
            } => {
                ui.label("Antwort am Tokenlimit beendet.");
            }
            _ => {
                ui.label("Chatverlauf bleibt nur in dieser App-Sitzung.");
            }
        }
        let ready = self.enabled
            && self.worker.is_none()
            && self.models.iter().any(|m| m.name == self.selected);
        let edit = ui.add_enabled(
            ready,
            egui::TextEdit::multiline(&mut self.draft)
                .desired_rows(2)
                .desired_width(f32::INFINITY)
                .char_limit(buddy_chat::MAX_INPUT_BYTES)
                .hint_text("Schreib Beni eine Nachricht …"),
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
        edit.on_hover_text("Senden: Cmd+Enter auf macOS, Ctrl+Enter auf Windows/Linux. Enter fügt eine neue Zeile ein.");
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    ready && !self.draft.trim().is_empty(),
                    egui::Button::new("Senden"),
                )
                .clicked()
                || shortcut
            {
                self.send(&ctx);
            }
            if ui
                .add_enabled(self.worker.is_some(), egui::Button::new("Abbrechen"))
                .clicked()
            {
                self.cancel();
            }
            if ui
                .add_enabled(self.worker.is_none(), egui::Button::new("Chat leeren"))
                .clicked()
            {
                self.session.clear();
                self.pending_input.clear();
            }
            if ui.button("Ollama trennen").clicked() {
                self.disconnect();
            }
            if ui.button("Schliessen").clicked() {
                self.close();
            }
        });
        if self.worker.is_some() {
            ui.ctx()
                .request_repaint_after_for(Duration::from_millis(100), egui::ViewportId::ROOT);
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
                ctx.request_repaint_of(id());
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
pub fn id() -> egui::ViewportId {
    egui::ViewportId::from_hash_of("beni-chat-window")
}
pub fn builder() -> egui::ViewportBuilder {
    egui::ViewportBuilder::default()
        .with_title("Mit Beni reden")
        .with_app_id("io.witzelfitz.beni-buddy.chat")
        .with_inner_size([640.0, 720.0])
        .with_min_inner_size([420.0, 520.0])
        .with_clamp_size_to_monitor_size(true)
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_mouse_passthrough(false)
        .with_window_level(egui::WindowLevel::Normal)
        .with_taskbar(true)
        .with_active(true)
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
