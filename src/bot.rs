use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use tracing::error;

use crate::gpt::ChatGPT;
use crate::{translate, Database, DbMessage};

#[derive(Clone)]
pub struct Bot {
    database: Database,
    gpt: Arc<Mutex<ChatGPT>>,
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
            gpt: Arc::new(Mutex::new(gpt)),
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
        let message_ru = match self.google_tl.translate(message, "en").await {
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
                message_en: message.to_string(),
                message_ru: message_ru.to_string(),
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

        // TODO: process message

        Some("Ok".to_string())
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
