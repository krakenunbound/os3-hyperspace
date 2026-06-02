use crate::message::{AgentMessage, AgentReply, AgentRole};

#[derive(Debug, Default)]
pub struct StubModel;

impl StubModel {
    pub fn complete(&self, message: AgentMessage) -> Result<AgentReply, &'static str> {
        let text = match message.role {
            AgentRole::System => {
                "Stub model received a system prompt. Real local inference plugs in here.".into()
            }
            AgentRole::User => format!(
                "Hyperspace agent (stub): I received your request — \"{}\". \
                 When the Redox-side runtime is wired up, this will run fully local.",
                truncate(&message.text, 120)
            ),
            AgentRole::Assistant => "Stub model echo.".into(),
        };

        Ok(AgentReply { text })
    }
}

fn truncate(input: &str, max: usize) -> String {
    if input.chars().count() <= max {
        return input.to_string();
    }
    let shortened: String = input.chars().take(max.saturating_sub(1)).collect();
    format!("{shortened}…")
}
