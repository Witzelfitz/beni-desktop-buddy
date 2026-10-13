//! User-local IPC adapter. The deterministic core owns validation and queue policy.
mod endpoint;
mod transport;
pub use endpoint::Endpoint;
pub use transport::{Ack, ClientError, MAX_RECENT_MESSAGES, ReceivedNotification, Server, notify};
