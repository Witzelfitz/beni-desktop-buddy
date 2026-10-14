use crate::*;
use std::{collections::VecDeque, fmt};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Idle,
    Waiting,
    Speaking,
    Completed {
        reason: FinishReason,
        usage: Usage,
    },
    Cancelled,
    Failed(ChatError),
}

struct Turn {
    input: String,
    output: String,
    context_output: String,
}
impl Turn {
    fn bytes(&self) -> usize {
        self.input.len() + self.context_output.len()
    }
}
struct Active {
    id: RequestId,
    input: String,
    cancellation: CancellationToken,
    started_ms: u64,
    last_text_ms: Option<u64>,
    events: usize,
}

/// One conversation, one active request. History is session-only and contains
/// complete user/assistant pairs exclusively. All time is passed explicitly.
#[derive(Default)]
pub struct ChatSession {
    next_id: u64,
    active: Option<Active>,
    phase: Phase,
    response: String,
    reply: Option<Reply>,
    history: VecDeque<Turn>,
}
impl fmt::Debug for ChatSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChatSession")
            .field("phase", &self.phase)
            .field("history_turns", &self.history.len())
            .field("response_bytes", &self.response.len())
            .finish()
    }
}
impl Drop for ChatSession {
    fn drop(&mut self) {
        self.cancel();
    }
}
impl ChatSession {
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn response(&self) -> &str {
        &self.response
    }
    pub fn reply(&self) -> Option<&Reply> {
        self.reply.as_ref()
    }
    pub fn history_turns(&self) -> usize {
        self.history.len()
    }
    /// Complete pairs for the visible chat/context preview, oldest first.
    pub fn history(&self) -> impl DoubleEndedIterator<Item = (&str, &str)> {
        self.history
            .iter()
            .map(|turn| (turn.input.as_str(), turn.output.as_str()))
    }
    pub fn busy(&self) -> bool {
        self.active.is_some()
    }

    /// Exact bounded messages for a visible preview, without starting a request.
    pub fn preview(&self, input: &str) -> Result<Vec<Message>, ChatError> {
        // Check raw bytes first: surrounding whitespace cannot bypass the limit.
        if input.len() > MAX_INPUT_BYTES {
            return Err(ChatError::InputTooLong);
        }
        if !contract::valid_text(input) {
            return Err(ChatError::InvalidInput);
        }
        let input = input.trim();
        if input.is_empty() {
            return Err(ChatError::EmptyInput);
        }
        let mut bytes = PERSONA.len() + input.len();
        let mut recent = Vec::new();
        for turn in self.history.iter().rev() {
            if bytes + turn.bytes() > MAX_CONTEXT_BYTES {
                break;
            }
            bytes += turn.bytes();
            recent.push(turn);
        }
        let mut messages = vec![Message {
            role: Role::System,
            text: PERSONA.into(),
        }];
        for turn in recent.into_iter().rev() {
            messages.push(Message {
                role: Role::User,
                text: turn.input.clone(),
            });
            messages.push(Message {
                role: Role::Assistant,
                text: turn.context_output.clone(),
            });
        }
        messages.push(Message {
            role: Role::User,
            text: input.into(),
        });
        Ok(messages)
    }

    /// No provider call is made here. The UI obtains this request only after an
    /// explicit send, then hands it and the token to its background adapter.
    pub fn start(
        &mut self,
        model: &str,
        input: &str,
        now_ms: u64,
    ) -> Result<(ChatRequest, CancellationToken), ChatError> {
        if self.busy() {
            return Err(ChatError::Busy);
        }
        let messages = self.preview(input)?;
        let input = input.trim();
        let model = model.trim();
        if !valid_model_name(model) {
            return Err(ChatError::InvalidModel);
        }
        self.next_id = self.next_id.checked_add(1).ok_or(ChatError::Protocol)?;
        let id = RequestId(self.next_id);
        let cancellation = CancellationToken::default();
        self.active = Some(Active {
            id,
            input: input.into(),
            cancellation: cancellation.clone(),
            started_ms: now_ms,
            last_text_ms: None,
            events: 0,
        });
        self.response.clear();
        self.reply = None;
        self.phase = Phase::Waiting;
        Ok((
            ChatRequest {
                id,
                model: model.into(),
                messages,
            },
            cancellation,
        ))
    }

    /// Returns false for late events from a completed/cancelled/previous request.
    /// The UI polls its channel without blocking and supplies monotonic time.
    pub fn receive(&mut self, id: RequestId, event: StreamEvent, now_ms: u64) -> bool {
        if !self.active.as_ref().is_some_and(|a| a.id == id) {
            return false;
        }
        self.advance(now_ms);
        let Some(active) = self.active.as_mut() else {
            return false;
        };
        if active.cancellation.is_cancelled() {
            self.cancel();
            return false;
        }
        active.events += 1;
        if active.events > MAX_STREAM_EVENTS {
            self.fail(ChatError::StreamLimit);
            return true;
        }
        match event {
            StreamEvent::Progress => {
                active.last_text_ms = Some(now_ms);
            }
            StreamEvent::Reply(reply) => {
                if !self.response.is_empty() || self.reply.is_some() {
                    self.fail(ChatError::Protocol);
                } else {
                    self.response = reply.text();
                    self.reply = Some(reply);
                    active.last_text_ms = Some(now_ms);
                    self.phase = Phase::Speaking;
                }
            }
            StreamEvent::Delta(text) => {
                if self.reply.is_some()
                    || text.len() > MAX_DELTA_BYTES
                    || !contract::valid_text(&text)
                {
                    self.fail(ChatError::InvalidResponse);
                } else if text.len() > MAX_RESPONSE_BYTES - self.response.len() {
                    self.fail(ChatError::ResponseLimit);
                } else if !text.is_empty() {
                    self.response.push_str(&text);
                    active.last_text_ms = Some(now_ms);
                    self.phase = Phase::Speaking;
                }
            }
            StreamEvent::Finished { reason, usage } => {
                if self.reply.is_none() {
                    match crate::reply::standard_text(&self.response) {
                        Ok(text) => self.response = text,
                        Err(error) => {
                            self.response.clear();
                            self.fail(error);
                            return true;
                        }
                    }
                }
                if self.response.trim().is_empty() {
                    self.fail(ChatError::InvalidResponse);
                } else {
                    let active = self.active.take().expect("active request checked above");
                    self.history.push_back(Turn {
                        input: active.input,
                        output: self.response.clone(),
                        context_output: self
                            .reply
                            .as_ref()
                            .map(Reply::encode)
                            .unwrap_or_else(|| self.response.clone()),
                    });
                    while self.history.len() > MAX_HISTORY_TURNS
                        || self.history.iter().map(Turn::bytes).sum::<usize>() > MAX_CONTEXT_BYTES
                    {
                        self.history.pop_front();
                    }
                    self.phase = Phase::Completed { reason, usage };
                }
            }
            StreamEvent::Failed(error) => {
                if error == ChatError::Cancelled {
                    self.cancel();
                } else {
                    self.fail(error);
                }
            }
        }
        true
    }

    pub fn advance(&mut self, now_ms: u64) {
        if let Some(active) = &self.active {
            if active.cancellation.is_cancelled() {
                self.cancel();
            } else if now_ms.saturating_sub(active.started_ms) >= REQUEST_TIMEOUT_MS
                || active
                    .last_text_ms
                    .is_some_and(|last| now_ms.saturating_sub(last) >= IDLE_TIMEOUT_MS)
            {
                self.fail(ChatError::Timeout);
            }
        }
    }

    pub fn cancel(&mut self) {
        if let Some(active) = self.active.take() {
            active.cancellation.cancel();
            self.phase = Phase::Cancelled;
        }
    }

    /// Also cancels a pending request. IDs are not reset: late worker events
    /// cannot become part of a new conversation after clearing.
    pub fn clear(&mut self) {
        self.cancel();
        self.history.clear();
        self.response.clear();
        self.reply = None;
        self.phase = Phase::Idle;
    }

    fn fail(&mut self, error: ChatError) {
        if let Some(active) = self.active.take() {
            active.cancellation.cancel();
        }
        self.phase = Phase::Failed(error);
    }
}
