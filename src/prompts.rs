use askama::Template;
use chrono::Utc;
use itertools::Itertools;

use crate::database::{DbSummary, DbUser};
use crate::gpt::{GptMessage, GptRole};
use crate::{Database, DbMessage};

#[derive(Template)]
#[template(path = "chat.txt")]
pub struct ChatSystem<'a> {
    pub users: &'a [DbUser],
    pub date: &'a str,
    pub time: &'a str,
    pub summary: &'a str,
    pub messages: &'a [DbMessage],
}

pub const CHAT_USER_PROMPT: &str = include_str!("../templates/chat_user.txt");
pub const CHAT_SUMMARY_PROMPT: &str = include_str!("../templates/summary_user.txt");

async fn get_system_prompt(
    database: &Database,
    channel: u64,
    min_count: i64,
) -> anyhow::Result<(String, usize)> {
    let DbSummary {
        summary,
        last_update,
        ..
    } = database.get_summary(channel).await?.unwrap_or_default();

    let messages = database
        .get_messages(channel, last_update, min_count)
        .await?;

    let mut users = messages
        .iter()
        .filter(|m| m.sender != "Kasumi")
        .map(|m| m.sender.to_string())
        .unique()
        .collect::<Vec<_>>();
    users.push("Kasumi".to_string());

    let users = database.get_users(&users).await?;

    let now = Utc::now();
    let date = now.format("%e %B %Y").to_string();
    let time = now.format("%r").to_string();

    Ok((
        ChatSystem {
            users: &users[..],
            date: &date,
            time: &time,
            summary: &summary,
            messages: &messages[..],
        }
        .render()?,
        messages.len(),
    ))
}

pub async fn get_prompt(
    database: &Database,
    channel_id: u64,
    user_prompt: &str,
    min_count: i64,
) -> anyhow::Result<(Vec<GptMessage>, usize)> {
    let (system_prompt, message_count) = get_system_prompt(database, channel_id, min_count).await?;
    let gpt_request = vec![
        GptMessage {
            role: GptRole::System,
            content: system_prompt,
        },
        GptMessage {
            role: GptRole::User,
            content: user_prompt.to_string(),
        },
    ];
    Ok((gpt_request, message_count))
}
