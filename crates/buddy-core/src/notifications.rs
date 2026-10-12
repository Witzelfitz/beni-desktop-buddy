//! Bounded status protocol and scheduling. The caller supplies monotonic milliseconds.
use crate::{BuddyError, Engine};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const MAX_WIRE_BYTES: usize = 4096;
pub const MAX_TEXT_CHARS: usize = 280;
pub const MAX_QUEUE: usize = 20;
pub const ALLOWED_STATES: &[&str] = &[
    "daumen",
    "jubeln",
    "stopp",
    "traurig",
    "nachdenken",
    "ueberrascht",
    "idee",
    "reden",
    "hallo",
];

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    #[default]
    Normal,
    High,
}
fn default_ttl() -> u16 {
    30
}
fn default_source() -> String {
    "cli".into()
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Notification {
    pub v: u8,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default = "default_ttl")]
    pub ttl_s: u16,
}
impl Notification {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            v: 1,
            text: text.into(),
            state: None,
            source: default_source(),
            priority: Priority::Normal,
            ttl_s: default_ttl(),
        }
    }
    pub fn parse(bytes: &[u8], engine: &Engine) -> Result<Self, BuddyError> {
        if bytes.len() > MAX_WIRE_BYTES {
            return Err(BuddyError("Nachricht grösser als 4 KiB".into()));
        }
        let message: Self = serde_json::from_slice(bytes)
            .map_err(|_| BuddyError("Ungültiges Nachrichten-JSON".into()))?;
        message.validate(engine)
    }
    pub fn validate(mut self, engine: &Engine) -> Result<Self, BuddyError> {
        if self.v != 1 {
            return Err(BuddyError("Unbekannte Protokollversion".into()));
        }
        if !(1..=120).contains(&self.ttl_s) {
            return Err(BuddyError(
                "TTL muss zwischen 1 und 120 Sekunden liegen".into(),
            ));
        }
        if self.source.is_empty()
            || self.source.len() > 48
            || !self
                .source
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        {
            return Err(BuddyError("Quelle: 1–48 ASCII-Zeichen aus Buchstaben, Zahlen, Punkt, Unterstrich oder Bindestrich".into()));
        }
        self.text = self
            .text
            .chars()
            .filter(|c| !c.is_control())
            .collect::<String>()
            .trim()
            .to_owned();
        if self.text.is_empty() || self.text.chars().count() > MAX_TEXT_CHARS {
            return Err(BuddyError("Text muss 1–280 Zeichen enthalten".into()));
        }
        let state = match self.state.as_deref() {
            Some(name) => engine.normalize(name)?,
            None => RuleClassifier.classify(&self.text).into(),
        };
        if !ALLOWED_STATES.contains(&state.as_str()) {
            return Err(BuddyError(
                "Animation für Statusmeldungen nicht erlaubt".into(),
            ));
        }
        self.state = Some(state);
        Ok(self)
    }
}

pub trait EmotionClassifier {
    fn classify(&self, text: &str) -> &'static str;
}
pub struct RuleClassifier;
impl EmotionClassifier for RuleClassifier {
    fn classify(&self, text: &str) -> &'static str {
        let mut text = text.to_lowercase();
        // Strip neutral error phrases, but still recognise a separate failure in the message.
        let mut neutral_success = false;
        for phrase in ["keine fehler", "no errors", "error-free", "fehlerfrei"] {
            if text.contains(phrase) {
                neutral_success = true;
                text = text.replace(phrase, " ");
            }
        }
        let words: Vec<_> = text.split(|c: char| !c.is_alphanumeric()).collect();
        for (state, keys) in [
            (
                "stopp",
                &[
                    "failed",
                    "failure",
                    "error",
                    "fehler",
                    "fehlgeschlagen",
                    "abgebrochen",
                ][..],
            ),
            (
                "ueberrascht",
                &["approval", "permission", "freigabe", "aufmerksamkeit"][..],
            ),
            (
                "daumen",
                &[
                    "success",
                    "successful",
                    "erfolgreich",
                    "fertig",
                    "passed",
                    "done",
                    "grün",
                    "completed",
                ][..],
            ),
            (
                "nachdenken",
                &[
                    "running",
                    "läuft",
                    "building",
                    "compiling",
                    "prüfe",
                    "kompiliere",
                ][..],
            ),
        ] {
            if keys.iter().any(|key| words.contains(key)) {
                return state;
            }
        }
        if text.contains("braucht dich") || text.contains("needs attention") {
            return "ueberrascht";
        }
        if neutral_success { "daumen" } else { "reden" }
    }
}
#[derive(Debug)]
pub struct Delivery {
    pub message: Notification,
    pub remaining_ms: u64,
}
struct Entry {
    message: Notification,
    expires_ms: u64,
}
#[derive(Default)]
pub struct NotificationQueue {
    entries: VecDeque<Entry>,
    receipts: VecDeque<(u64, String)>,
}
impl NotificationQueue {
    fn prune(&mut self, now_ms: u64) {
        self.entries.retain(|e| e.expires_ms > now_ms);
        self.receipts
            .retain(|(time, _)| now_ms.saturating_sub(*time) < 10_000);
    }
    /// Accepted messages have already been validated. Ack means queued, not displayed.
    pub fn enqueue(&mut self, message: Notification, now_ms: u64) -> Result<(), BuddyError> {
        self.prune(now_ms);
        if self.receipts.len() >= 20
            || self
                .receipts
                .iter()
                .filter(|(_, s)| s == &message.source)
                .count()
                >= 5
        {
            return Err(BuddyError(
                "Zu viele Meldungen: 5 pro Quelle / 20 insgesamt in 10 Sekunden".into(),
            ));
        }
        if self.entries.len() >= MAX_QUEUE {
            return Err(BuddyError("Warteschlange voll".into()));
        }
        let expires_ms = now_ms.saturating_add(u64::from(message.ttl_s) * 1000);
        self.receipts.push_back((now_ms, message.source.clone()));
        self.entries.push_back(Entry {
            message,
            expires_ms,
        });
        Ok(())
    }
    pub fn pop(&mut self, now_ms: u64) -> Option<Delivery> {
        self.prune(now_ms);
        let index = self
            .entries
            .iter()
            .position(|e| e.message.priority == Priority::High)
            .unwrap_or(0);
        let entry = self.entries.remove(index)?;
        Some(Delivery {
            remaining_ms: entry.expires_ms.saturating_sub(now_ms),
            message: entry.message,
        })
    }
    pub fn len(&mut self, now_ms: u64) -> usize {
        self.prune(now_ms);
        self.entries.len()
    }
    pub fn is_empty(&mut self, now_ms: u64) -> bool {
        self.len(now_ms) == 0
    }
}
/// Read at roughly 15 characters/second, with a short minimum and a bounded maximum.
pub fn reading_time_ms(text: &str) -> u64 {
    (3_000 + text.chars().count() as u64 * 65).clamp(4_000, 30_000)
}
