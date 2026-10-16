use std::process::Stdio;

use serde_json::Value;
use tokio::process::Command;

use crate::tool::{ToolArgs, ToolResult, tool_arg, tool_spec};

#[derive(Debug)]
pub struct BashTool;

impl BashTool {
    pub async fn call(&self, args: ToolArgs<'_>) -> ToolResult {
        let command_line = tool_arg!(args->command);

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

    pub fn spec() -> &'static Value {
        tool_spec!({
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
        })
    }
}
