use std::{
    fmt::{Debug, Display},
    path::Path,
};

use anyhow::anyhow;
use serde_json::{Value, json};
use tokio::{fs, io};

// TODO: turn ToolId into Tool enum with tool values as variants (i.e. Read(ReadTool) etc.)
#[derive(Debug)]
pub enum ToolId {
    Read,
}

impl TryFrom<&str> for ToolId {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Read" => Ok(ToolId::Read),
            _ => Err(anyhow!("Unknown tool id: {value}")),
        }
    }
}

impl Display for ToolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

pub struct Read;

impl Read {
    // TODO: later, turn into an instance method, if needed
    pub async fn call(file_path: impl AsRef<Path>) -> io::Result<String> {
        fs::read_to_string(file_path).await
    }

    pub fn spec() -> Value {
        json!({
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
        })
    }
}
