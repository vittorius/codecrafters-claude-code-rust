use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, from_str, json};
use std::{collections::HashMap, env, fmt::Write, process, sync::OnceLock};

use crate::{skill::Skill, tool::Tool};

use anyhow::{Result, anyhow};

mod skill;
mod tool;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

// TODO: try rewriting this challenge using `types` and/or *-types feature(s) from the openai-async crate

// TODO: turn into a Skill::all() method similar to the one of Tool
static SKILLS: OnceLock<HashMap<String, Skill>> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<()> {
    let skills = skill::build_skills().await?;
    SKILLS.get_or_init(move || skills);

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

async fn agent_loop(client: &Client<OpenAIConfig>, raw_prompt: &str) -> Result<()> {
    let mut messages: Vec<Value> =
        vec![json!({ "role": "system", "content": skills_advertisement()? })];
    messages.extend(
        prompt_messages(raw_prompt)
            .into_iter()
            .map(|msg| json!({ "role": "user", "content": msg })),
    );

    loop {
        // NOTE: we always have a single choice the response for this challenge (use `response["choices"][0]`)

        // eprintln!("request: {messages:#?}");
        let response: Value = send_request(client, &messages).await?;
        // eprintln!("response: {response:#?}");
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

                    let result = tool
                        .call(arguments, SKILLS.get().expect("Skills must be initialized"))
                        .await?;
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

fn prompt_messages(raw_prompt: &str) -> Vec<String> {
    let mut all = raw_prompt;
    let mut head;
    let mut tail;
    let mut skills = Vec::<&Skill>::new();
    loop {
        (head, tail) = all.split_once(' ').unwrap_or((all, ""));

        if head.starts_with('/')
            && let Some(skill) = crate::skills().get(&head[1..])
        {
            skills.push(skill);

            all = tail;
        } else {
            break;
        }
    }

    if skills.is_empty() {
        return vec![raw_prompt.to_owned()];
    }

    skills.into_iter().map(|skill| skill.body(all)).collect()
}

async fn send_request(client: &Client<OpenAIConfig>, messages: &[Value]) -> Result<Value> {
    client
        .chat()
        .create_byot(json!({
            "messages": messages,
            "model": "anthropic/claude-haiku-4.5",
            // "model": "nvidia/nemotron-3-ultra-550b-a55b:free",
            "tools": Vec::from_iter(Tool::all().iter().map(Tool::spec))
        }))
        .await
        .map_err(Into::into)
}

fn skills() -> &'static HashMap<String, Skill> {
    SKILLS.get().expect("Skills must be initialized")
}

fn skills_advertisement() -> Result<String> {
    let mut message = String::from("You have access to the following skills:\n\n");

    for skill in SKILLS.get().expect("Skills must be initialized").values() {
        writeln!(message, "- {}: {}", skill.name, skill.description)?;
    }

    writeln!(
        message,
        "If a skill matches the user's request, call the Skill tool with \
        its name and follow the instructions it returns."
    )?;

    Ok(message)
}
