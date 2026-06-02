use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub role: AgentRole,
    pub text: String,
}

impl AgentMessage {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: AgentRole::User,
            text: text.into(),
        }
    }

    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: AgentRole::System,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReply {
    pub text: String,
}
