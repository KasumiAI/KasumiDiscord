use serde::Deserialize;
use thiserror::Error;

const API_URL: &str = "https://translate.google.com/translate_a/single?client=at&dt=t&dj=1";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_2_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36";

#[derive(Debug, Deserialize)]
struct Sentence {
    trans: String,
}

#[derive(Debug, Deserialize)]
struct Response {
    sentences: Vec<Sentence>,
}

#[derive(Error, Debug)]
pub enum GoogleError {
    #[error("Failed to make request")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Failed to parse response")]
    ParseFailed(#[from] serde_json::Error),
    #[error("Translation is empty")]
    Empty,
}

#[derive(Clone)]
pub struct GoogleTranslate {
    client: reqwest::Client,
}

impl GoogleTranslate {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn translate(&self, text: &str, to: &str) -> Result<String, GoogleError> {
        let params = [("sl", "auto"), ("tl", to), ("q", text)];
        let res = self
            .client
            .post(API_URL)
            .form(&params)
            .header("User-Agent", USER_AGENT)
            .send()
            .await?
            .json::<Response>()
            .await?;

        let mut result = String::new();
        for sentence in res.sentences {
            result.push_str(&sentence.trans);
        }
        match result.len() {
            0 => Err(GoogleError::Empty),
            _ => Ok(result),
        }
    }
}
