use std::{
    fmt::{Debug, Display},
    path::Path,
    sync::LazyLock,
};

use anyhow::anyhow;
use anyhow::{Ok, Result};
use serde_json::{Map, Value, json};
use tokio::fs;

// TODO: turn ToolId into Tool enum with tool values as variants (i.e. Read(ReadTool) etc.)
#[derive(Debug)]
pub enum Tool {
    Read,
    Write,
}

impl Tool {
    pub async fn call(&self, arguments: &Map<String, Value>) -> Result<Value> {
        match self {
            Tool::Read => self.call_read_tool(arguments).await.map(Into::into),
            Tool::Write => self.call_write_tool(arguments).await.map(Into::into),
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
        }
    }

    pub fn all() -> &'static [Tool] {
        static ALL: LazyLock<Vec<Tool>> = LazyLock::new(|| {
            let mut tools = Vec::new();

            let read_added: () = tools.push(Tool::Read);
            let write_added: () = tools.push(Tool::Write);

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
                }
            }));

            tools
        });

        ALL.as_ref()
    }

    async fn call_read_tool(&self, arguments: &Map<String, Value>) -> Result<String> {
        // TODO: use declarative macro for this
        let path = Path::new(
            arguments["file_path"]
                .as_str()
                .ok_or_else(|| anyhow!("Failed to deserialize file_path argument"))?,
        );

        fs::read_to_string(path).await.map_err(Into::into)
    }

    async fn call_write_tool(&self, arguments: &Map<String, Value>) -> Result<String> {
        let path = Path::new(
            arguments["file_path"]
                .as_str()
                .ok_or_else(|| anyhow!("Failed to deserialize file_path argument"))?,
        );
        let contents = arguments["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Failed to deserialize content argument"))?;

        fs::write(path, contents).await?;

        Ok(contents.to_owned())
    }
}

impl TryFrom<&str> for Tool {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Read" => Ok(Tool::Read),
            "Write" => Ok(Tool::Write),
            _ => Err(anyhow!("Unknown tool: {value}")),
        }
    }
}

impl Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
