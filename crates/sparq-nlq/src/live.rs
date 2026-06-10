//! Live Anthropic backend (non-default `live` feature). Thin by design: one
//! blocking POST to the Messages API per [`Llm::complete`] call, no streaming, no
//! retries beyond reqwest's defaults — wrap it in [`crate::RecordingLlm`] to capture
//! fixtures for offline replay.
//!
//! Never used by CI or the test suite; compile-checked with
//! `cargo check -p sparq-nlq --features live`.

use crate::Llm;

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";
const DEFAULT_MODEL: &str = "claude-sonnet-4-6";

/// Calls the Anthropic Messages API (`POST /v1/messages`) with the prompt as a
/// single user message. The API key comes from `ANTHROPIC_API_KEY`.
pub struct AnthropicLlm {
    api_key: String,
    /// Model id (exact string, no date suffix). Default: `claude-sonnet-4-6`.
    pub model: String,
    /// Per-completion output cap. Generated SPARQL is short; 2048 is generous.
    pub max_tokens: u32,
    client: reqwest::blocking::Client,
}

/// Hard wall-clock cap on one API round trip — `Llm::complete` must never hang the
/// loop on a stalled connection.
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

impl AnthropicLlm {
    /// Default model, key from `ANTHROPIC_API_KEY`.
    pub fn from_env() -> Result<Self, String> {
        Self::with_model(DEFAULT_MODEL)
    }

    /// Specific model id, key from `ANTHROPIC_API_KEY`.
    pub fn with_model(model: impl Into<String>) -> Result<Self, String> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY is not set".to_string())?;
        let client = reqwest::blocking::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|e| format!("cannot build HTTP client: {e}"))?;
        Ok(Self {
            api_key,
            model: model.into(),
            max_tokens: 2048,
            client,
        })
    }
}

impl Llm for AnthropicLlm {
    fn complete(&self, prompt: &str) -> Result<String, String> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "messages": [{ "role": "user", "content": prompt }],
        });
        let resp = self
            .client
            .post(API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .json(&body)
            .send()
            .map_err(|e| format!("anthropic request failed: {e}"))?;
        let status = resp.status();
        let v: serde_json::Value = resp
            .json()
            .map_err(|e| format!("anthropic response was not JSON: {e}"))?;
        if !status.is_success() {
            return Err(format!(
                "anthropic API error ({status}): {}",
                v["error"]["message"].as_str().unwrap_or("unknown")
            ));
        }
        // The completion is the concatenated text blocks of the response content.
        let text: String = v["content"]
            .as_array()
            .map(|blocks| {
                blocks
                    .iter()
                    .filter(|b| b["type"] == "text")
                    .filter_map(|b| b["text"].as_str())
                    .collect()
            })
            .unwrap_or_default();
        if text.is_empty() {
            return Err("anthropic response carried no text content".to_string());
        }
        Ok(text)
    }
}
