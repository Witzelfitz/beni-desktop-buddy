//! Local-only Ollama transport. No proxy, redirect, cloud model or model pull.
use buddy_chat::{
    CancellationToken, ChatError, ChatProvider, ChatRequest, FinishReason, MAX_REPLY_BYTES,
    MAX_STREAM_EVENTS, Reply, StreamEvent, Usage, valid_model_name,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{future::Future, time::Duration};

const MAX_METADATA_BYTES: usize = 2 * 1024 * 1024;
const MAX_LINE_BYTES: usize = 64 * 1024;
const MAX_WIRE_BYTES: usize = 1024 * 1024;
const MAX_MODELS: usize = 64;

#[derive(Clone)]
pub(super) struct Model {
    pub name: String,
    pub size: u64,
    digest: String,
}

#[derive(Clone)]
pub(super) struct Ollama {
    endpoint: String,
    selected: Option<Model>,
}
impl Default for Ollama {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:11434".into(),
            selected: None,
        }
    }
}
impl Ollama {
    pub fn with_model(&self, model: Model) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            selected: Some(model),
        }
    }
    pub fn discover(&self, cancellation: &CancellationToken) -> Result<Vec<Model>, ChatError> {
        run(cancellation, Duration::from_secs(20), async {
            let client = client()?;
            let tags: Tags =
                decode(metadata(client.get(format!("{}/api/tags", self.endpoint))).await?)?;
            if tags.models.len() > MAX_MODELS {
                return Err(ChatError::Protocol);
            }
            let mut models = Vec::new();
            for tag in tags.models {
                if !tag.valid_local() {
                    continue;
                }
                let info: Info = decode(
                    metadata(
                        client
                            .post(format!("{}/api/show", self.endpoint))
                            .json(&json!({"model":tag.name})),
                    )
                    .await?,
                )?;
                if info.local_chat() {
                    models.push(Model {
                        name: tag.name,
                        size: tag.size,
                        digest: tag.digest,
                    });
                }
            }
            models.sort_by(|a, b| a.size.cmp(&b.size).then(a.name.cmp(&b.name)));
            models.dedup_by(|a, b| a.name == b.name);
            Ok(models)
        })
    }
}
impl ChatProvider for Ollama {
    fn stream(
        &self,
        request: &ChatRequest,
        cancellation: &CancellationToken,
        emit: &mut dyn FnMut(StreamEvent) -> Result<(), ChatError>,
    ) -> Result<(), ChatError> {
        let selected = self
            .selected
            .as_ref()
            .filter(|m| m.name == request.model())
            .ok_or(ChatError::InvalidModel)?;
        run(
            cancellation,
            Duration::from_millis(request.timeout_ms()),
            async {
                let client = client()?;
                // Re-check the selected local model before sending any conversation.
                let tags: Tags =
                    decode(metadata(client.get(format!("{}/api/tags", self.endpoint))).await?)?;
                if tags.models.len() > MAX_MODELS
                    || !tags.models.iter().any(|m| {
                        m.valid_local() && m.name == selected.name && m.digest == selected.digest
                    })
                {
                    return Err(ChatError::InvalidModel);
                }
                let info: Info = decode(
                    metadata(
                        client
                            .post(format!("{}/api/show", self.endpoint))
                            .json(&json!({"model":selected.name})),
                    )
                    .await?,
                )?;
                if !info.local_chat() {
                    return Err(ChatError::InvalidModel);
                }
                let messages = request.messages().iter().map(|m| json!({
                "role": match m.role() { buddy_chat::Role::System => "system", buddy_chat::Role::User => "user", buddy_chat::Role::Assistant => "assistant" },
                "content": m.text(),
            })).collect::<Vec<_>>();
                let mut response = client.post(format!("{}/api/chat", self.endpoint))
                .json(&json!({"model":selected.name,"messages":messages,"stream":true,"format":buddy_chat::reply_schema(),"options":{"num_predict":request.max_output_tokens(),"temperature":0.2}}))
                .send().await.map_err(network_error)?;
                status(response.status())?;
                let mut decoder = Decoder::new(&selected.name);
                while let Some(chunk) = response.chunk().await.map_err(network_error)? {
                    if cancellation.is_cancelled() {
                        return Err(ChatError::Cancelled);
                    }
                    if decoder.push(&chunk, emit)? {
                        return Ok(());
                    }
                }
                decoder.finish(emit)
            },
        )
    }
}

fn client() -> Result<reqwest::Client, ChatError> {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_millis(buddy_chat::REQUEST_TIMEOUT_MS))
        .pool_max_idle_per_host(0)
        .build()
        .map_err(|_| ChatError::Unavailable)
}
fn network_error(error: reqwest::Error) -> ChatError {
    if error.is_timeout() {
        ChatError::Timeout
    } else {
        ChatError::Unavailable
    }
}
fn status(status: reqwest::StatusCode) -> Result<(), ChatError> {
    match status.as_u16() {
        200 => Ok(()),
        401 | 403 => Err(ChatError::Authentication),
        429 => Err(ChatError::RateLimited),
        404 => Err(ChatError::Unavailable),
        300..=399 => Err(ChatError::Protocol),
        _ => Err(ChatError::ProviderFailure),
    }
}
async fn metadata(request: reqwest::RequestBuilder) -> Result<Vec<u8>, ChatError> {
    let mut response = request
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(network_error)?;
    status(response.status())?;
    if response
        .content_length()
        .is_some_and(|n| n > MAX_METADATA_BYTES as u64)
    {
        return Err(ChatError::Protocol);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(network_error)? {
        if chunk.len() > MAX_METADATA_BYTES - bytes.len() {
            return Err(ChatError::Protocol);
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
fn decode<T: serde::de::DeserializeOwned>(bytes: Vec<u8>) -> Result<T, ChatError> {
    serde_json::from_slice(&bytes).map_err(|_| ChatError::Protocol)
}
fn run<T>(
    cancellation: &CancellationToken,
    timeout: Duration,
    future: impl Future<Output = Result<T, ChatError>>,
) -> Result<T, ChatError> {
    if cancellation.is_cancelled() {
        return Err(ChatError::Cancelled);
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| ChatError::Unavailable)?;
    runtime.block_on(async {
        tokio::select! {
            biased;
            _ = async { while !cancellation.is_cancelled() { tokio::time::sleep(Duration::from_millis(25)).await; } } => Err(ChatError::Cancelled),
            result = tokio::time::timeout(timeout, future) => result.unwrap_or(Err(ChatError::Timeout)),
        }
    })
}

#[derive(Deserialize)]
struct Tags {
    models: Vec<Tag>,
}
#[derive(Deserialize)]
struct Tag {
    name: String,
    size: u64,
    digest: String,
    #[serde(default)]
    remote_model: String,
    #[serde(default)]
    remote_host: String,
}
impl Tag {
    fn valid_local(&self) -> bool {
        valid_model_name(&self.name)
            && !self.name.to_ascii_lowercase().contains("cloud")
            && self.size > 0
            && self.digest.len() == 64
            && self.digest.bytes().all(|b| b.is_ascii_hexdigit())
            && self.remote_host.is_empty()
            && self.remote_model.is_empty()
    }
}
#[derive(Deserialize)]
struct Info {
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    remote_host: String,
    #[serde(default)]
    remote_model: String,
}
impl Info {
    fn local_chat(&self) -> bool {
        self.remote_host.is_empty()
            && self.remote_model.is_empty()
            && self.capabilities.iter().any(|c| c == "completion")
    }
}
#[derive(Deserialize)]
struct WireEvent {
    #[serde(default)]
    error: Option<Value>,
    #[serde(default)]
    model: String,
    #[serde(default)]
    remote_host: String,
    #[serde(default)]
    remote_model: String,
    #[serde(default)]
    message: Option<WireMessage>,
    #[serde(default)]
    done: Option<bool>,
    #[serde(default)]
    done_reason: String,
    #[serde(default)]
    prompt_eval_count: Option<u64>,
    #[serde(default)]
    eval_count: Option<u64>,
}
#[derive(Deserialize)]
struct WireMessage {
    role: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    tool_calls: Vec<Value>,
}
struct Decoder {
    model: String,
    line: Vec<u8>,
    wire_bytes: usize,
    reply: String,
    events: usize,
    done: bool,
}
impl Decoder {
    fn new(model: &str) -> Self {
        Self {
            model: model.into(),
            line: Vec::new(),
            wire_bytes: 0,
            reply: String::new(),
            events: 0,
            done: false,
        }
    }
    fn push(
        &mut self,
        bytes: &[u8],
        emit: &mut dyn FnMut(StreamEvent) -> Result<(), ChatError>,
    ) -> Result<bool, ChatError> {
        if bytes.len() > MAX_WIRE_BYTES - self.wire_bytes {
            return Err(ChatError::Protocol);
        }
        self.wire_bytes += bytes.len();
        for byte in bytes {
            if *byte == b'\n' {
                self.event(emit)?;
                self.line.clear();
                if self.done {
                    return Ok(true);
                }
            } else {
                if self.line.len() >= MAX_LINE_BYTES {
                    return Err(ChatError::Protocol);
                }
                self.line.push(*byte);
            }
        }
        Ok(false)
    }
    fn finish(
        &mut self,
        emit: &mut dyn FnMut(StreamEvent) -> Result<(), ChatError>,
    ) -> Result<(), ChatError> {
        if !self.line.is_empty() {
            self.event(emit)?;
            self.line.clear();
        }
        if self.done {
            Ok(())
        } else {
            Err(ChatError::Protocol)
        }
    }
    fn event(
        &mut self,
        emit: &mut dyn FnMut(StreamEvent) -> Result<(), ChatError>,
    ) -> Result<(), ChatError> {
        if self.line.iter().all(u8::is_ascii_whitespace) {
            return Ok(());
        }
        self.events += 1;
        if self.events > MAX_STREAM_EVENTS {
            return Err(ChatError::StreamLimit);
        }
        let event: WireEvent =
            serde_json::from_slice(&self.line).map_err(|_| ChatError::Protocol)?;
        if event.error.is_some() {
            return Err(ChatError::ProviderFailure);
        }
        if event.model != self.model
            || !event.remote_host.is_empty()
            || !event.remote_model.is_empty()
        {
            return Err(ChatError::Protocol);
        }
        let done = event.done.ok_or(ChatError::Protocol)?;
        if let Some(message) = event.message {
            if message.role != "assistant" || !message.tool_calls.is_empty() {
                return Err(ChatError::Protocol);
            }
            if message.content.len() > MAX_REPLY_BYTES - self.reply.len() {
                return Err(ChatError::ResponseLimit);
            }
            if !message.content.is_empty() {
                self.reply.push_str(&message.content);
                emit(StreamEvent::Progress)?;
            }
        } else if !done {
            return Err(ChatError::Protocol);
        }
        if done {
            let reason = match event.done_reason.as_str() {
                "" | "stop" => FinishReason::Stop,
                "length" => FinishReason::Length,
                _ => return Err(ChatError::Protocol),
            };
            let reply = Reply::decode(&self.reply).map_err(|error| {
                if reason == FinishReason::Length && error == ChatError::InvalidResponse {
                    ChatError::ResponseLimit
                } else {
                    error
                }
            })?;
            emit(StreamEvent::Reply(reply))?;
            emit(StreamEvent::Finished {
                reason,
                usage: Usage {
                    input_tokens: event.prompt_eval_count,
                    output_tokens: event.eval_count,
                },
            })?;
            self.done = true;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
