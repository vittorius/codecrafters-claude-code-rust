use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, from_str, json};
use std::{env, process};

use crate::tool::Tool;

use anyhow::{Result, anyhow};

mod tool;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

// TODO: try rewriting this challenge using `types` and/or *-types feature(s) from the openai-async crate

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

    agent_loop(&client, &args.prompt).await?;

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    Ok(())
}

async fn agent_loop(client: &Client<OpenAIConfig>, prompt: &str) -> Result<()> {
    let mut messages: Vec<Value> = vec![json!(
        {
            "role": "user",
            "content": prompt
        }
    )];

    loop {
        // NOTE: we always have a single choice the response for this challenge (use `response["choices"][0]`)

        let response: Value = send_request(client, &messages).await?;
        messages.push(response["choices"][0]["message"].clone());

        if let Some(tool_calls) = response["choices"][0]["message"]["tool_calls"].as_array()
            && !tool_calls.is_empty()
        {
            for tool_call in tool_calls {
                if let Some(call_type) = tool_call["type"].as_str()
                    && call_type == "function"
                {
                    let tool = Tool::try_from(
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

                    let result = tool.call(arguments).await?;
                    messages.push(json!(
                        {
                          "role": "tool",
                          "tool_call_id": tool_call["id"],
                          "content": result
                        }
                    ));
                }
            }
        } else if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
            println!("{content}");
            break;
        }
    }

    Ok(())
}

async fn send_request(client: &Client<OpenAIConfig>, messages: &[Value]) -> Result<Value> {
    client
        .chat()
        .create_byot(json!({
            "messages": messages,
            "model": "anthropic/claude-haiku-4.5",
            "tools": Vec::from_iter(Tool::all().iter().map(Tool::spec))
        }))
        .await
        .map_err(Into::into)
}
