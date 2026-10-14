//! Validated conversational text and a fixed emotion vocabulary. No system actions.
use crate::{ChatError, MAX_RESPONSE_BYTES, contract::valid_text};
use serde::{Deserialize, Serialize};
use std::fmt;

pub const MAX_REPLY_BYTES: usize = 24 * 1024;
pub const MAX_SEGMENTS: usize = 6;
pub const MAX_SEGMENT_CHARS: usize = 600;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Emotion {
    Neutral,
    Friendly,
    Curious,
    Happy,
    Sad,
    Surprised,
    Encouraging,
    Thoughtful,
}
impl Emotion {
    pub const ALL: [Self; 8] = [
        Self::Neutral,
        Self::Friendly,
        Self::Curious,
        Self::Happy,
        Self::Sad,
        Self::Surprised,
        Self::Encouraging,
        Self::Thoughtful,
    ];

    /// These are existing SDK animations. The model never supplies an animation name.
    pub fn animation(self) -> &'static str {
        match self {
            Self::Neutral => "reden",
            Self::Friendly => "hallo",
            Self::Curious => "idee",
            Self::Happy => "jubeln",
            Self::Sad => "traurig",
            Self::Surprised => "ueberrascht",
            Self::Encouraging => "daumen",
            Self::Thoughtful => "nachdenken",
        }
    }
}

#[derive(Clone, Serialize)]
pub struct Segment {
    text: String,
    emotion: Emotion,
}
impl Segment {
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn emotion(&self) -> Emotion {
        self.emotion
    }
    pub fn reading_ms(&self) -> u64 {
        (2500 + self.text.chars().count() as u64 * 55).clamp(4500, 30_000)
    }
}
impl fmt::Debug for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Segment")
            .field("bytes", &self.text.len())
            .field("emotion", &self.emotion)
            .finish()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Reply {
    v: u8,
    segments: Vec<Segment>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireReply {
    v: u8,
    segments: Vec<WireSegment>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSegment {
    text: String,
    emotion: Emotion,
}

impl Reply {
    pub fn decode(json: &str) -> Result<Self, ChatError> {
        if json.len() > MAX_REPLY_BYTES {
            return Err(ChatError::ResponseLimit);
        }
        let wire: WireReply = serde_json::from_str(json).map_err(|_| ChatError::InvalidResponse)?;
        if wire.v != 1 || wire.segments.is_empty() || wire.segments.len() > MAX_SEGMENTS {
            return Err(ChatError::InvalidResponse);
        }
        let mut bytes = 0;
        let mut segments = Vec::new();
        for segment in wire.segments {
            let text = standard_text(&segment.text)?;
            if text.chars().count() > MAX_SEGMENT_CHARS {
                return Err(ChatError::InvalidResponse);
            }
            bytes += text.len();
            if bytes > MAX_RESPONSE_BYTES {
                return Err(ChatError::ResponseLimit);
            }
            segments.push(Segment {
                text,
                emotion: segment.emotion,
            });
        }
        Ok(Self { v: 1, segments })
    }
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }
    pub fn text(&self) -> String {
        self.segments
            .iter()
            .map(Segment::text)
            .collect::<Vec<_>>()
            .join("\n\n")
    }
    pub fn encode(&self) -> String {
        serde_json::to_string(self).expect("validated primitive reply serialization")
    }
}

pub fn reply_schema() -> serde_json::Value {
    serde_json::json!({
        "type":"object", "additionalProperties":false, "required":["v","segments"],
        "properties":{
            "v":{"type":"integer","const":1},
            "segments":{"type":"array","minItems":1,"maxItems":MAX_SEGMENTS,"items":{
                "type":"object","additionalProperties":false,"required":["text","emotion"],
                "properties":{
                    "text":{"type":"string","minLength":1,"maxLength":MAX_SEGMENT_CHARS},
                    "emotion":{"type":"string","enum":Emotion::ALL}
                }
            }}
        }
    })
}

pub(crate) fn standard_text(raw: &str) -> Result<String, ChatError> {
    let text = raw.trim().replace('ß', "ss").replace('ẞ', "SS");
    if text.is_empty() || !valid_text(&text) {
        return Err(ChatError::InvalidResponse);
    }
    // A conservative lexical guard catches common dialect greetings/words.
    // The fixed persona supplies the broader language instruction.
    if text.split(|c: char| !c.is_alphabetic()).any(|word| {
        matches!(
            word.to_lowercase().as_str(),
            "hoi"
                | "grüezi"
                | "grüessech"
                | "sali"
                | "häsch"
                | "hesch"
                | "isch"
                | "chli"
                | "nöd"
                | "nid"
                | "gäll"
                | "chasch"
                | "chunsch"
                | "chum"
                | "wotsch"
        )
    }) {
        return Err(ChatError::Language);
    }
    Ok(text)
}
