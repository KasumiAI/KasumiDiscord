use serde::Deserialize;
use thiserror::Error;

const API_URL: &str = "https://api-free.deepl.com/v2/translate";

#[derive(Debug, Deserialize)]
struct Translation {
    text: String,
}

#[derive(Debug, Deserialize)]
struct Response {
    translations: Vec<Translation>,
}

#[derive(Error, Debug)]
pub enum DeepLError {
    #[error("Failed to make request")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Failed to parse response")]
    ParseFailed(#[from] serde_json::Error),
    #[error("Translation is empty")]
    Empty,
}

pub struct DeepLTranslate {
    client: reqwest::Client,
    key: String,
}

impl DeepLTranslate {
    pub fn new(key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            key: key.to_string(),
        }
    }

    pub async fn translate(&self, text: &str, to: &str) -> Result<String, DeepLError> {
        let params = [
            ("target_lang", to),
            ("text", text),
            ("formality", "prefer_less"),
        ];
        let res = self
            .client
            .post(API_URL)
            .header("Authorization", format!("DeepL-Auth-Key {}", self.key))
            .form(&params)
            .send()
            .await?
            .json::<Response>()
            .await?;

        let mut result = String::new();
        for sentence in res.translations {
            result.push_str(&sentence.text);
        }
        match result.len() {
            0 => Err(DeepLError::Empty),
            _ => Ok(result),
        }
    }
}
