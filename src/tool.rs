use std::{
    fmt::{Debug, Display},
    sync::LazyLock,
};

use anyhow::anyhow;
use anyhow::{Ok, Result};
use serde_json::{Map, Value};

use crate::tool::{bash::BashTool, read::ReadTool, skill::SkillTool, write::WriteTool};

mod bash;
mod read;
mod skill;
mod write;

macro_rules! tool_arg {
    ($map:ident -> $param:tt) => {
        $map[stringify!($param)].as_str().ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to deserialize \"{0}\" tool argument",
                stringify!($param)
            )
        })?
    };
}
pub(crate) use tool_arg;

macro_rules! tool_arg_or_default {
    ($map:ident -> $param:tt) => {
        $map.get(stringify!($param)).and_then(|v| v.as_str()).unwrap_or("")
    };
}
pub(crate) use tool_arg_or_default;

macro_rules! tool_spec {
    ($($json:tt)+) => {{
        use std::sync::LazyLock;

        static SPEC: LazyLock<Value> = LazyLock::new(|| {
            serde_json::json!($($json)+)
        });

        &*SPEC
    }};
}
pub(crate) use tool_spec;

#[derive(Debug)]
pub enum Tool {
    Read(ReadTool),
    Write(WriteTool),
    Bash(BashTool),
    Skill(SkillTool),
}

type ToolArgs<'a> = &'a Map<String, Value>;
pub type ToolResult = Result<String>;

impl Tool {
    pub fn spec(&self) -> &'static Value {
        match self {
            Tool::Read(_) => ReadTool::spec(),
            Tool::Write(_) => WriteTool::spec(),
            Tool::Bash(_) => BashTool::spec(),
            Tool::Skill(_) => SkillTool::spec(),
        }
    }

    pub fn all() -> &'static [Tool] {
        static ALL: LazyLock<Vec<Tool>> = LazyLock::new(|| {
            let mut tools = Vec::new();

            let read_added: () = tools.push(Tool::Read(ReadTool));
            let write_added: () = tools.push(Tool::Write(WriteTool));
            let bash_added: () = tools.push(Tool::Bash(BashTool));
            let skill_added: () = tools.push(Tool::Skill(SkillTool));

            debug_assert!(tools.iter().all(|value| {
                match value {
                    Tool::Read(_) => {
                        let () = read_added;
                        true
                    }
                    Tool::Write(_) => {
                        let () = write_added;
                        true
                    }
                    Tool::Bash(_) => {
                        let () = bash_added;
                        true
                    }
                    Tool::Skill(_) => {
                        let () = skill_added;
                        true
                    }
                }
            }));

            tools
        });

        ALL.as_ref()
    }
}

impl TryFrom<&str> for Tool {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Read" => Ok(Tool::Read(ReadTool)),
            "Write" => Ok(Tool::Write(WriteTool)),
            "Bash" => Ok(Tool::Bash(BashTool)),
            "Skill" => Ok(Tool::Skill(SkillTool)),
            _ => Err(anyhow!("Unknown tool: {value}")),
        }
    }
}

impl Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
