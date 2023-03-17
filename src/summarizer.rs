use std::time::Duration;

use regex::Regex;
use tracing::{info, warn};

use crate::gpt::ChatGPT;
use crate::prompts::{get_prompt, CHAT_SUMMARY_PROMPT};
use crate::Database;

pub struct Summarizer {
    gpt: ChatGPT,
    database: Database,
}

pub async fn summarize_now(gpt: &ChatGPT, database: &Database) {
    info!("Summarizing channels");
    let channels = match database.channel_list().await {
        Ok(channels) => channels,
        Err(e) => {
            warn!("Failed to get channels: {:?}", e);
            return;
        }
    };

    for channel in channels {
        if let Err(e) = process_channel(gpt, database, channel).await {
            warn!("Failed to process channel: {:?}", e);
        }
    }
}

async fn process_channel(
    gpt: &ChatGPT,
    database: &Database,
    channel_id: u64,
) -> anyhow::Result<()> {
    let (prompt, message_count) = get_prompt(database, channel_id, CHAT_SUMMARY_PROMPT, 0).await?;
    if message_count == 0 {
        info!("No messages for channel {}", channel_id);
        return Ok(());
    }

    info!("Generating summary for channel {}", channel_id);
    let gpt_response = gpt.send(&prompt, 0.4).await?;

    let re_summary = Regex::new(r"SUMMARY (.+?) END").unwrap();
    if let Some(cap) = re_summary.captures(&gpt_response.message.content) {
        if let Some(summary) = cap.get(1) {
            if let Err(e) = database
                .update_summary(channel_id, summary.as_str().trim())
                .await
            {
                warn!("Failed to update summary: {:?}", e);
            } else {
                info!("Updated summary for channel {}", channel_id);
            }
        }
    }

    let re_info = Regex::new(r"USER (.+?) INFO (.+?) END").unwrap();
    for cap in re_info.captures_iter(&gpt_response.message.content) {
        let user = {
            match cap.get(1) {
                Some(summary) => summary.as_str().trim(),
                None => continue,
            }
        };
        let info = {
            match cap.get(2) {
                Some(summary) => summary.as_str().trim(),
                None => continue,
            }
        };

        if user.to_lowercase() == "kasumi" {
            continue;
        }

        if let Err(e) = database.update_user(user, info).await {
            warn!("Failed to update user info: {:?}", e);
        } else {
            info!("Updated user info for user {}", user);
        }
    }
    Ok(())
}

impl Summarizer {
    pub fn new(gpt: ChatGPT, database: Database) -> Self {
        Self { gpt, database }
    }

    pub async fn start(&self) {
        loop {
            tokio::time::sleep(Duration::from_secs(5 * 60)).await;
            summarize_now(&self.gpt, &self.database).await;
        }
    }
}
