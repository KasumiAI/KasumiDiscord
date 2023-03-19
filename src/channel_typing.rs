use std::collections::HashMap;
use std::sync::Arc;

use serenity::http::{Http, Typing};

struct TypingData {
    count: usize,
    typing: Typing,
}

pub struct TypingManager {
    typing_data: HashMap<u64, TypingData>,
}

impl TypingManager {
    pub fn new() -> Self {
        Self {
            typing_data: HashMap::new(),
        }
    }

    pub fn start_typing(&mut self, channel_id: u64, http: Arc<Http>) {
        match self.typing_data.get_mut(&channel_id) {
            Some(typing_data) => {
                typing_data.count += 1;
            }
            None => {
                if let Ok(typing) = Typing::start(http, channel_id) {
                    self.typing_data
                        .insert(channel_id, TypingData { count: 1, typing });
                }
            }
        }
    }

    pub fn stop_typing(&mut self, channel_id: u64) {
        if let Some(typing_data) = self.typing_data.remove(&channel_id) {
            if typing_data.count == 1 {
                let _ = typing_data.typing.stop();
            } else {
                self.typing_data.insert(
                    channel_id,
                    TypingData {
                        count: typing_data.count - 1,
                        typing: typing_data.typing,
                    },
                );
            }
        }
    }
}
