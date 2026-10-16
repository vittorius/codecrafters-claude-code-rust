use std::collections::HashMap;

use anyhow::anyhow;
use serde_json::Value;

use crate::message::Message;
use crate::tool::{tool_arg, tool_spec};
use crate::{
    skill::Skill,
    tool::{ToolArgs, ToolResult, tool_arg_or_default},
};

#[derive(Debug)]
pub struct SkillTool;

impl SkillTool {
    pub async fn call(
        &self,
        args: ToolArgs<'_>,
        available_skills: &HashMap<String, Skill>,
        agent_loop_fn: impl AsyncFn(Vec<Message>) -> ToolResult,
    ) -> ToolResult {
        let name = tool_arg!(args->name);
        let args = tool_arg_or_default!(args->args);

        let skill = available_skills
            .get(name)
            .ok_or_else(|| anyhow!("Missing skill {name} (invoked by the Skill tool)"))?;

        let body = skill.body(args);

        if skill.in_subagent() {
            agent_loop_fn(vec![Message::User(body)]).await
        } else {
            Ok(body)
        }
    }

    pub fn spec() -> &'static Value {
        tool_spec!({
            "type": "function",
                "function": {
                    "name": "Skill",
                    "description": "Load a skill's instructions into the conversation",
                    "parameters": {
                    "type": "object",
                    "required": ["name"],
                    "properties": {
                        "name": { "type": "string", "description": "The name of the skill to use" },
                        "args": { "type": "string", "description": "Optional arguments for the skill" }
                    }
                }
            }
        })
    }
}
