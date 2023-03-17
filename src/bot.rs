use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use regex::Regex;
use tokio::sync::Mutex;
use tracing::error;

use crate::gpt::{ChatGPT, GptFinishReason};
use crate::prompts::{get_prompt, CHAT_USER_PROMPT};
use crate::summarizer::summarize_now;
use crate::{translate, Database, DbMessage};

#[derive(Clone)]
pub struct Bot {
    database: Database,
    gpt: ChatGPT,
    google_tl: translate::GoogleTranslate,
    deepl_tl: translate::DeepLTranslate,
    channel_last: Arc<Mutex<HashMap<u64, u64>>>,
}

impl Bot {
    pub fn new(
        database: Database,
        gpt: ChatGPT,
        google_tl: translate::GoogleTranslate,
        deepl_tl: translate::DeepLTranslate,
    ) -> Self {
        Self {
            database,
            gpt,
            google_tl,
            deepl_tl,
            channel_last: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn process_message(
        &mut self,
        channel_id: u64,
        sender: &str,
        message: &str,
    ) -> Option<String> {
        // translate message to english
        let message_en = match self.google_tl.translate(message, "en").await {
            Ok(message) => message,
            Err(e) => {
                error!("Failed to translate message: {:?}", e);
                return None;
            }
        };

        // add message to database
        if let Err(e) = self
            .database
            .add_message(&DbMessage {
                channel: channel_id.to_string(),
                sender: sender.to_string(),
                message_en: message_en.to_string(),
                message_ru: message.to_string(),
                date_time: Utc::now().naive_utc(),
            })
            .await
        {
            error!("Failed to add message to database: {:?}", e);
            return None;
        }

        // wait for new message
        if self.has_new(channel_id).await {
            return None;
        }

        // Make GPT prompt
        let (gpt_request, _) =
            match get_prompt(&self.database, channel_id, CHAT_USER_PROMPT, 10).await {
                Ok(request) => request,
                Err(e) => {
                    error!("Failed to generate GPT prompt: {:?}", e);
                    return None;
                }
            };

        // Send GPT request
        let gpt_response = match self.gpt.send(&gpt_request, 0.4).await {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to generate GPT response: {:?}", e);
                return None;
            }
        };

        // Check for too many tokens
        if gpt_response.usage.total_tokens > 3000
            || gpt_response.finish_reason == GptFinishReason::Length
        {
            summarize_now(&self.gpt, &self.database).await;
        }

        // Parse GPT response
        let response_en = self.parse_response(gpt_response.message.content.as_str())?;
        let response_ru = match self.deepl_tl.translate(&response_en, "RU").await {
            Ok(message) => message,
            Err(e) => {
                error!("Failed to translate response: {:?}", e);
                return None;
            }
        };

        // Put response to database
        if let Err(e) = self
            .database
            .add_message(&DbMessage {
                channel: channel_id.to_string(),
                sender: "Kasumi".to_string(),
                message_en: response_en,
                message_ru: response_ru.clone(),
                date_time: Utc::now().naive_utc(),
            })
            .await
        {
            error!("Failed to put response to database: {:?}", e);
            return None;
        }

        Some(response_ru)
    }

    fn parse_response(&self, response: &str) -> Option<String> {
        let re = Regex::new(r"USER (.+?) SAYS (.+?) END").unwrap();
        let caps = re.captures(response)?;
        let user = caps.get(1)?.as_str().trim().to_lowercase();
        if user != "kasumi" {
            return None;
        }
        let message = caps.get(2)?.as_str().trim().to_string();
        Some(message)
    }

    async fn has_new(&mut self, channel_id: u64) -> bool {
        let last = {
            let mut map = self.channel_last.lock().await;
            let last = map.entry(channel_id).or_insert(0);
            *last += 1;
            *last
        };
        let old = last;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let last = {
            let mut map = self.channel_last.lock().await;
            *map.entry(channel_id).or_insert(0)
        };
        old != last
    }
}
