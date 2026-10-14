use buddy_chat::{Emotion, Reply, RequestId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    Thinking,
    Speaking,
    Emotion(RequestId, usize, Emotion),
}
impl Cue {
    pub fn animation(self) -> &'static str {
        match self {
            Self::Thinking => "nachdenken",
            Self::Speaking => "reden",
            Self::Emotion(_, _, emotion) => emotion.animation(),
        }
    }
    pub fn one_shot(self) -> bool {
        matches!(self, Self::Emotion(_, _, emotion) if emotion != Emotion::Neutral)
    }
}
pub struct Presentation {
    id: RequestId,
    reply: Reply,
    index: usize,
    until_ms: u64,
    active: bool,
}
impl Presentation {
    pub fn new(id: RequestId, reply: Reply, now_ms: u64) -> Self {
        let until_ms = now_ms.saturating_add(reply.segments()[0].reading_ms());
        Self {
            id,
            reply,
            index: 0,
            until_ms,
            active: true,
        }
    }
    pub fn text(&self) -> &str {
        self.reply.segments()[self.index].text()
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn parts(&self) -> usize {
        self.reply.segments().len()
    }
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn has_next(&self) -> bool {
        self.index + 1 < self.parts()
    }
    pub fn cue(&self) -> Option<Cue> {
        self.active.then_some(Cue::Emotion(
            self.id,
            self.index,
            self.reply.segments()[self.index].emotion(),
        ))
    }
    pub fn next(&mut self, now_ms: u64) {
        if self.has_next() {
            self.index += 1;
            self.until_ms = now_ms.saturating_add(self.reply.segments()[self.index].reading_ms());
            self.active = true;
        } else {
            self.active = false;
        }
    }
    pub fn advance(&mut self, now_ms: u64) {
        if self.active && now_ms >= self.until_ms {
            self.next(now_ms);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn segments_keep_their_text_and_emotion_then_release_the_avatar() {
        let mut session = buddy_chat::ChatSession::default();
        let (request, _) = session.start("test", "Frage", 0).unwrap();
        let reply=Reply::decode(r#"{"v":1,"segments":[{"text":"Hallo!","emotion":"friendly"},{"text":"Eine gute Idee!","emotion":"happy"}]}"#).unwrap();
        let mut p = Presentation::new(request.id(), reply, 0);
        assert_eq!(p.cue().unwrap().animation(), "hallo");
        p.advance(4499);
        assert_eq!(p.index(), 0);
        p.advance(4500);
        assert_eq!(p.index(), 1);
        assert_eq!(p.text(), "Eine gute Idee!");
        assert_eq!(p.cue().unwrap().animation(), "jubeln");
        p.advance(9000);
        assert!(!p.active());
        assert!(p.cue().is_none());
        p.advance(u64::MAX);
        assert_eq!(p.index(), 1);
    }
}
