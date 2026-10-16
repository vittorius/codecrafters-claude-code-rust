use std::path::Path;

use serde_json::Value;
use tokio::fs;

use crate::tool::{ToolArgs, ToolResult, tool_arg, tool_spec};

#[derive(Debug)]
pub struct WriteTool;

impl WriteTool {
    pub async fn call(&self, args: ToolArgs<'_>) -> ToolResult {
        let path = Path::new(tool_arg!(args->file_path));
        let contents = tool_arg!(args->content);

        fs::write(path, contents).await?;

        Ok(contents.to_owned())
    }

    pub fn spec() -> &'static Value {
        tool_spec!({
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
        })
    }
}
