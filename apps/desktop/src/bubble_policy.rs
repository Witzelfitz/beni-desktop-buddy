//! Pure arbitration of transient speech and conversation gestures.
//! Queues, deadlines and cancellation remain with their respective producers.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Idle,
    Manual,
    Status,
    Conversation,
    History,
    Error,
}

#[derive(Clone, Copy, Default)]
pub struct State {
    pub visible: bool,
    pub quiet: bool,
    pub history: bool,
    pub conversation: bool,
    pub error: bool,
    pub speech_active: bool,
    pub status_active: bool,
}

impl State {
    pub fn owner(self) -> Owner {
        if self.error {
            Owner::Error
        } else if self.history {
            Owner::History
        } else if self.conversation {
            Owner::Conversation
        } else if self.speech_active {
            if self.status_active {
                Owner::Status
            } else {
                Owner::Manual
            }
        } else {
            Owner::Idle
        }
    }

    // IPC may replace a greeting or manual text, but never active status text.
    pub fn accepts_ipc(self) -> bool {
        self.visible && !self.quiet && matches!(self.owner(), Owner::Idle | Owner::Manual)
    }

    // After checking IPC first, timer and reminder notices wait for all speech.
    pub fn accepts_local(self) -> bool {
        self.accepts_ipc() && self.owner() == Owner::Idle
    }

    // Quiet mode only suppresses automatic notices, not an explicit conversation.
    pub fn animates_conversation(self) -> bool {
        self.visible && self.owner() == Owner::Conversation && !self.speech_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_combination_preserves_the_delivery_and_animation_contract() {
        for bits in 0..128 {
            let flag = |n| bits & (1 << n) != 0;
            let state = State {
                visible: flag(0),
                quiet: flag(1),
                history: flag(2),
                conversation: flag(3),
                error: flag(4),
                speech_active: flag(5),
                status_active: flag(6),
            };
            assert_eq!(
                state.accepts_ipc(),
                state.visible
                    && !state.quiet
                    && !state.history
                    && !state.conversation
                    && !state.error
                    && !(state.speech_active && state.status_active)
            );
            assert_eq!(
                state.accepts_local(),
                state.accepts_ipc() && !state.speech_active
            );
            assert_eq!(
                state.animates_conversation(),
                state.visible
                    && state.conversation
                    && !state.history
                    && !state.error
                    && !state.speech_active
            );
            assert!(!(state.accepts_ipc() && state.animates_conversation()));
        }
    }

    #[test]
    fn owners_have_explicit_precedence_and_expired_status_releases_the_bubble() {
        let mut state = State {
            visible: true,
            status_active: true,
            ..State::default()
        };
        assert_eq!(state.owner(), Owner::Idle);
        state.speech_active = true;
        assert_eq!(state.owner(), Owner::Status);
        state.status_active = false;
        assert_eq!(state.owner(), Owner::Manual);
        state.conversation = true;
        assert_eq!(state.owner(), Owner::Conversation);
        state.history = true;
        assert_eq!(state.owner(), Owner::History);
        state.error = true;
        assert_eq!(state.owner(), Owner::Error);
    }
}
