//! Provider-independent chat contract and bounded, session-only conversation.
//!
//! This crate has no transport, credentials, persistence, GUI or animation dependency.
//! Adapters run outside the GUI thread and translate their protocol into these events.
mod contract;
mod reply;
mod session;

pub use contract::*;
pub use reply::*;
pub use session::{ChatSession, Phase};
