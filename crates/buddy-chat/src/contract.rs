use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub const MAX_INPUT_BYTES: usize = 4 * 1024;
pub const MAX_RESPONSE_BYTES: usize = 16 * 1024;
pub const MAX_DELTA_BYTES: usize = 1024;
pub const MAX_CONTEXT_BYTES: usize = 32 * 1024;
pub const MAX_HISTORY_TURNS: usize = 10;
pub const MAX_STREAM_EVENTS: usize = 4096;
pub const MAX_OUTPUT_TOKENS: u32 = 512;
pub const REQUEST_TIMEOUT_MS: u64 = 120_000;
pub const IDLE_TIMEOUT_MS: u64 = 30_000;

/// Fixed application persona, separate from status notifications and provider output.
pub const PERSONA: &str = "Du bist Beni, ein freundlicher Desktop-Buddy. Antworte kurz in Schweizer Hochdeutsch. Verwende ss statt Eszett. Du kannst keine Dateien öffnen, Programme ausführen oder den Bildschirm sehen. Behaupte keine ausgeführten Aktionen.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestId(pub(crate) u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Message {
    pub(crate) role: Role,
    pub(crate) text: String,
}
impl Message {
    pub fn role(&self) -> Role {
        self.role
    }
    pub fn text(&self) -> &str {
        &self.text
    }
}
impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Message")
            .field("role", &self.role)
            .field("bytes", &self.text.len())
            .finish()
    }
}

/// Constructed only by ChatSession after validation. Never contains an API key.
pub struct ChatRequest {
    pub(crate) id: RequestId,
    pub(crate) model: String,
    pub(crate) messages: Vec<Message>,
}
impl ChatRequest {
    pub fn id(&self) -> RequestId {
        self.id
    }
    pub fn model(&self) -> &str {
        &self.model
    }
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }
    pub fn max_output_tokens(&self) -> u32 {
        MAX_OUTPUT_TOKENS
    }
    pub fn timeout_ms(&self) -> u64 {
        REQUEST_TIMEOUT_MS
    }
}
impl fmt::Debug for ChatRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChatRequest")
            .field("id", &self.id)
            .field("model_bytes", &self.model.len())
            .field("messages", &self.messages)
            .finish()
    }
}

#[derive(Clone, Debug, Default)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinishReason {
    Stop,
    Length,
}

/// Adapters map provider errors to these safe categories. No raw response body,
/// URL, request text or credentials may be included in errors or diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatError {
    EmptyInput,
    InputTooLong,
    InvalidInput,
    InvalidModel,
    Busy,
    InvalidResponse,
    ResponseLimit,
    StreamLimit,
    Timeout,
    Unavailable,
    Authentication,
    RateLimited,
    ProviderFailure,
    Protocol,
    Cancelled,
}
impl fmt::Display for ChatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyInput => "Bitte gib eine Nachricht ein.",
            Self::InputTooLong => "Die Nachricht ist zu lang (höchstens 4 KiB).",
            Self::InvalidInput => "Die Nachricht enthält ungültige Steuerzeichen.",
            Self::InvalidModel => "Bitte wähle einen gültigen Modellnamen.",
            Self::Busy => "Eine Antwort läuft bereits.",
            Self::InvalidResponse => "Der Anbieter hat eine ungültige Antwort geliefert.",
            Self::ResponseLimit => "Die Antwort hat die Längengrenze erreicht.",
            Self::StreamLimit => "Die Antwort enthält zu viele Teilereignisse.",
            Self::Timeout => "Die Antwort hat zu lange gedauert.",
            Self::Unavailable => "Der Anbieter ist nicht erreichbar.",
            Self::Authentication => "Der Anbieter hat die Zugangsdaten abgelehnt.",
            Self::RateLimited => "Der Anbieter erlaubt gerade keine weitere Anfrage.",
            Self::ProviderFailure => "Der Anbieter konnte die Anfrage nicht bearbeiten.",
            Self::Protocol => "Die Antwort des Anbieters ist unvollständig oder ungültig.",
            Self::Cancelled => "Die Anfrage wurde abgebrochen.",
        })
    }
}
impl std::error::Error for ChatError {}

pub enum StreamEvent {
    Delta(String),
    Finished { reason: FinishReason, usage: Usage },
    Failed(ChatError),
}
impl fmt::Debug for StreamEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Delta(text) => f.debug_struct("Delta").field("bytes", &text.len()).finish(),
            Self::Finished { reason, usage } => f
                .debug_struct("Finished")
                .field("reason", reason)
                .field("usage", usage)
                .finish(),
            Self::Failed(error) => f.debug_tuple("Failed").field(error).finish(),
        }
    }
}

/// Run on a background worker, never on the GUI thread. Implementations must:
/// - set connect/read/overall transport timeouts and honour cancellation;
/// - bound wire buffers before decoding and split text into MAX_DELTA_BYTES;
/// - stop if emit fails, and emit exactly one terminal event on success;
/// - treat EOF without a terminal event as ChatError::Protocol;
/// - never expose reasoning, tools or raw provider errors as user-visible text.
///
/// The caller uses a bounded event channel; backpressure must remain cancellable.
pub trait ChatProvider: Send + Sync {
    fn stream(
        &self,
        request: &ChatRequest,
        cancellation: &CancellationToken,
        emit: &mut dyn FnMut(StreamEvent) -> Result<(), ChatError>,
    ) -> Result<(), ChatError>;
}

pub(crate) fn valid_text(text: &str) -> bool {
    !text
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\t')
}
