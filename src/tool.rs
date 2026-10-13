use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    path::Path,
    process::Stdio,
    sync::LazyLock,
};

use anyhow::anyhow;
use anyhow::{Ok, Result};
use serde_json::{Map, Value, json};
use tokio::{fs, process::Command};

use crate::skill::Skill;

macro_rules! tool_arg {
    ($map:ident -> $param:expr) => {
        $map[$param]
            .as_str()
            .ok_or_else(|| anyhow!("Failed to deserialize \"{0}\" tool argument", $param))?
    };
}

macro_rules! tool_arg_or_default {
    ($map:ident -> $param:expr) => {
        $map.get($param).and_then(|v| v.as_str()).unwrap_or("")
    };
}

#[derive(Debug)]
pub enum Tool {
    Read,
    Write,
    Bash,
    Skill,
}

type ToolArgs = Map<String, Value>;

impl Tool {
    pub async fn call(
        &self,
        arguments: &ToolArgs,
        available_skills: &HashMap<String, Skill>,
    ) -> Result<String> {
        match self {
            Tool::Read => self.call_read_tool(arguments).await,
            Tool::Write => self.call_write_tool(arguments).await,
            Tool::Bash => self.call_bash_tool(arguments).await,
            Tool::Skill => self.call_skill_tool(arguments, available_skills),
        }
    }

    pub fn spec(&self) -> Value {
        match self {
            Tool::Read => json!({
                "type": "function",
                    "function": {
                        "name": "Read",
                        "description": "Read and return the contents of a file",
                        "parameters": {
                            "type": "object",
                            "properties": {
                            "file_path": {
                                "type": "string",
                                "description": "The path to the file to read"
                            }
                        },
                        "required": ["file_path"]
                    }
                }
            }),
            Tool::Write => json!({
              "type": "function",
              "function": {
                "name": "Write",
                "description": "Write content to a file",
                "parameters": {
                  "type": "object",
                  "required": ["file_path", "content"],
                  "properties": {
                    "file_path": {
                      "type": "string",
                      "description": "The path of the file to write to"
                    },
                    "content": {
                      "type": "string",
                      "description": "The content to write to the file"
                    }
                  }
                }
              }
            }),
            Tool::Bash => json!( {
                  "type": "function",
                  "function": {
                    "name": "Bash",
                    "description": "Execute a shell command",
                    "parameters": {
                      "type": "object",
                      "required": ["command"],
                      "properties": {
                        "command": {
                          "type": "string",
                          "description": "The command to execute"
                        }
                      }
                    }
                  }
                } ),
            Tool::Skill => json!( { "type": "function",
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
                  } ),
        }
    }

    pub fn all() -> &'static [Tool] {
        static ALL: LazyLock<Vec<Tool>> = LazyLock::new(|| {
            let mut tools = Vec::new();

            let read_added: () = tools.push(Tool::Read);
            let write_added: () = tools.push(Tool::Write);
            let bash_added: () = tools.push(Tool::Bash);
            let skill_added: () = tools.push(Tool::Skill);

            debug_assert!(tools.iter().all(|value| {
                match value {
                    Tool::Read => {
                        let () = read_added;
                        true
                    }
                    Tool::Write => {
                        let () = write_added;
                        true
                    }
                    Tool::Bash => {
                        let () = bash_added;
                        true
                    }
                    Tool::Skill => {
                        let () = skill_added;
                        true
                    }
                }
            }));

            tools
        });

        ALL.as_ref()
    }

    async fn call_read_tool(&self, arguments: &ToolArgs) -> Result<String> {
        let path = Path::new(tool_arg!(arguments->"file_path"));

        fs::read_to_string(path).await.map_err(Into::into)
    }

    async fn call_write_tool(&self, arguments: &ToolArgs) -> Result<String> {
        let path = Path::new(tool_arg!(arguments->"file_path"));
        let contents = tool_arg!(arguments->"content");

        fs::write(path, contents).await?;

        Ok(contents.to_owned())
    }

    async fn call_bash_tool(&self, arguments: &ToolArgs) -> Result<String> {
        let command_line = tool_arg!(arguments->"command");

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(command_line);

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;
        let mut out_string = String::new();

        out_string.push_str(&String::from_utf8(output.stdout)?);
        out_string.push_str(&String::from_utf8(output.stderr)?);

        Ok(out_string)
    }

    #[allow(clippy::unused_self)]
    fn call_skill_tool(
        &self,
        arguments: &ToolArgs,
        available_skills: &HashMap<String, Skill>,
    ) -> Result<String> {
        let name = tool_arg!(arguments->"name");
        let args = tool_arg_or_default!(arguments->"args");

        let skill = available_skills
            .get(name)
            .ok_or_else(|| anyhow!("Missing skill {name} (invoked by the Skill tool)"))?;

        Ok(skill.body(args))
    }
}

impl TryFrom<&str> for Tool {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Read" => Ok(Tool::Read),
            "Write" => Ok(Tool::Write),
            "Bash" => Ok(Tool::Bash),
            "Skill" => Ok(Tool::Skill),
            _ => Err(anyhow!("Unknown tool: {value}")),
        }
    }
}

impl Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
