use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::debug;

#[derive(Debug, Serialize, Deserialize)]
pub enum GptRole {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Debug, Deserialize)]
pub enum GptFinishReason {
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "length")]
    Length,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GptMessage {
    pub role: GptRole,
    pub content: String,
}

pub struct ChatGPT {
    key: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct GptResponse {
    error: Option<GptError>,
    usage: Option<GptUsage>,
    choices: Option<Vec<GptChoice>>,
}

#[derive(Debug, Deserialize)]
pub struct GptError {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub param: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GptUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

#[derive(Debug, Deserialize)]
struct GptChoice {
    message: GptMessage,
    finish_reason: GptFinishReason,
}

#[derive(Error, Debug)]
pub enum ChatGPTError {
    #[error("Failed to make request")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Api returned an error: {0:?}")]
    RequestError(GptError),
    #[error("Failed to parse response")]
    ParseFailed(#[from] serde_json::Error),
    #[error("Something went wrong")]
    Unknown,
}

#[derive(Debug)]
pub struct GptReply {
    pub message: GptMessage,
    pub usage: GptUsage,
    pub finish_reason: GptFinishReason,
}

#[derive(Debug, Serialize)]
struct GptRequest<'s> {
    model: &'static str,
    messages: &'s [GptMessage],
    temperature: f32,
}

impl ChatGPT {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn send(
        &self,
        messages: &[GptMessage],
        temperature: f32,
    ) -> Result<GptReply, ChatGPTError> {
        let request = GptRequest {
            model: "gpt-3.5-turbo",
            messages,
            temperature,
        };

        debug!("GPT Sending request: {:?}", request);

        let resp = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .json(&request)
            .header("Authorization", format!("Bearer {}", self.key))
            .send()
            .await?
            .json::<GptResponse>()
            .await?;

        debug!("GPT response: {:?}", resp);

        match resp {
            GptResponse {
                usage: Some(usage),
                choices: Some(mut choices),
                error: None,
            } => {
                let choice = choices.pop().ok_or(ChatGPTError::Unknown)?;
                Ok(GptReply {
                    message: choice.message,
                    usage,
                    finish_reason: choice.finish_reason,
                })
            }
            GptResponse {
                usage: _,
                choices: _,
                error: Some(error),
            } => Err(ChatGPTError::RequestError(error)),
            _ => Err(ChatGPTError::Unknown),
        }
    }
}
