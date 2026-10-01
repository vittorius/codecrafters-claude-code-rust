use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Map, Value, from_str, json};
use std::{env, path::Path, process};

use crate::tool::{Read, ToolId};

use anyhow::{Result, anyhow};

mod tool;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);

    #[allow(unused_variables)]
    let response: Value = client
        .chat()
        .create_byot(json!({
            "messages": [
                {
                    "role": "user",
                    "content": args.prompt
                }
            ],
            "model": "anthropic/claude-haiku-4.5",
            "tools": [
               Read::spec()
            ]
        }))
        .await?;

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    if let Some(tool_calls) = response["choices"][0]["message"]["tool_calls"].as_array() {
        for tool_call in tool_calls {
            if let Some(call_type) = tool_call["type"].as_str()
                && call_type == "function"
            {
                let tool_id = ToolId::try_from(
                    tool_call["function"]["name"]
                        .as_str()
                        .ok_or_else(|| anyhow!("Failed to deserialize tool name"))?,
                )?;
                let arguments = tool_call["function"]["arguments"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Failed to deserialize tool arguments"))?;
                let arguments = from_str::<Value>(arguments)?;
                let arguments = arguments
                    .as_object()
                    .ok_or_else(|| anyhow!("Failed to parse tool arguments"))?;

                let result = call_tool(tool_id, arguments).await?;
                println!("{result}");
            }
        }
    } else if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
        println!("{content}");
    }

    Ok(())
}

// TODO: may change the String result type later
async fn call_tool(tool_id: ToolId, arguments: &Map<String, Value>) -> Result<String> {
    match tool_id {
        ToolId::Read => {
            // TODO: think of moving params parsing inside ::call
            Read::call(Path::new(arguments["file_path"].as_str().ok_or_else(
                || anyhow!("Failed to deserialize file_path argument"),
            )?))
            .await
            .map_err(Into::into)
        }
    }
}
