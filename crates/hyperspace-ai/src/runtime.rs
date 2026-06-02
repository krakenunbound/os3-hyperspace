use hyperspace_core::{HyperspaceError, Result};

use crate::message::{AgentMessage, AgentReply};
use crate::stub::StubModel;

pub trait AgentRuntime {
    fn ping(&self) -> Result<String>;
    fn complete(&self, message: AgentMessage) -> Result<AgentReply>;
}

#[derive(Debug, Default)]
pub struct LocalAgentRuntime {
    model: StubModel,
}

impl LocalAgentRuntime {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AgentRuntime for LocalAgentRuntime {
    fn ping(&self) -> Result<String> {
        Ok("Local agent runtime online (stub model)".into())
    }

    fn complete(&self, message: AgentMessage) -> Result<AgentReply> {
        self.model
            .complete(message)
            .map_err(|err| HyperspaceError::AiRuntime(err.to_string()))
    }
}
