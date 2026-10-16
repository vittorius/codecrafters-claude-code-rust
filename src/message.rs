use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "role", content = "content", rename_all = "lowercase")]
pub enum Message {
    System(String),
    User(String),
}