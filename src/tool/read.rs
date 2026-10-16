use std::path::Path;

use serde_json::Value;
use tokio::fs;

use crate::tool::{ToolArgs, ToolResult, tool_arg};
use crate::tool::tool_spec;

#[derive(Debug)]
pub struct ReadTool;

impl ReadTool {
    pub async fn call(&self, args: ToolArgs<'_>) -> ToolResult {
        let path = Path::new(tool_arg!(args->file_path));

        fs::read_to_string(path).await.map_err(Into::into)
    }

    pub fn spec() -> &'static Value {
        tool_spec!({
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
