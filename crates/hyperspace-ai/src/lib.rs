//! AI runtime hooks for OS/3 Hyperspace.
//!
//! This crate defines the interface local agents will use. The current
//! implementation is a deterministic stub so the shell can be exercised
//! on Windows before Redox integration lands.

mod message;
mod runtime;
mod stub;

pub use message::{AgentMessage, AgentReply, AgentRole};
pub use runtime::{AgentRuntime, LocalAgentRuntime};
pub use stub::StubModel;
