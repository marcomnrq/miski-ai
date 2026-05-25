use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::ai::prompts;
use crate::models::ModelInfo;

/// HTTP client for Ollama's REST API with streaming support.
pub struct OllamaClient {
    pub base_url: String,
    pub model: String,
    client: Client,
}

/// Ollama API response for `/api/tags`.
#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    size: u64,
}

/// Ollama chat request body.
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// Ollama streaming chunk.
#[derive(Debug, Deserialize)]
struct ChatChunk {
    message: Option<ChunkMessage>,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct ChunkMessage {
    content: String,
}

/// Ollama pull request.
#[derive(Debug, Serialize)]
struct PullRequest {
    name: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct PullChunk {
    status: String,
    #[serde(default)]
    completed: Option<u64>,
    #[serde(default)]
    total: Option<u64>,
}

impl OllamaClient {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            client: Client::new(),
        }
    }

    /// Check if Ollama server is reachable.
    pub async fn health_check(&self) -> Result<bool, String> {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// List available models from Ollama.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>, String> {
        let url = format!("{}/api/tags", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Ollama returned status: {}", resp.status()));
        }

        let tags: TagsResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        let models = tags
            .models
            .into_iter()
            .map(|m| {
                let size_gb = m.size as f64 / 1_073_741_824.0;
                ModelInfo {
                    name: m.name,
                    size: format!("{:.1} GB", size_gb),
                    description: String::new(),
                    installed: true,
                }
            })
            .collect();

        Ok(models)
    }

    /// Pull a model from Ollama registry with streaming progress.
    pub async fn pull_model(
        &self,
        name: &str,
        app: tauri::AppHandle,
    ) -> Result<(), String> {
        let url = format!("{}/api/pull", self.base_url);
        let body = PullRequest {
            name: name.to_string(),
            stream: true,
        };

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to pull model: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Ollama pull failed: {}", resp.status()));
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete JSON lines
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim().to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                if let Ok(pull_chunk) = serde_json::from_str::<PullChunk>(&line) {
                    let percent = match (pull_chunk.completed, pull_chunk.total) {
                        (Some(done), Some(total)) if total > 0 => {
                            ((done as f64 / total as f64) * 100.0) as u32
                        }
                        _ => 0,
                    };

                    let _ = app.emit(
                        "model-pull-progress",
                        crate::models::PullProgress {
                            status: pull_chunk.status.clone(),
                            percent,
                        },
                    );

                    if pull_chunk.status == "success" {
                        let _ = app.emit("model-pull-complete", ());
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate a streaming summary of a meeting transcript.
    ///
    /// Emits `summary-chunk` events with text chunks and `summary-complete` when done.
    /// Returns the full accumulated summary text.
    pub async fn summarize_streaming(
        &self,
        transcript: &str,
        duration_minutes: u32,
        language: &str,
        notes: Option<&str>,
        app: tauri::AppHandle,
    ) -> Result<String, String> {
        let prompt =
            prompts::build_summary_prompt(transcript, duration_minutes, language, notes);
        self.chat_streaming(&prompt, "summary-chunk", "summary-complete", app)
            .await
    }

    /// Generate a title for a meeting from its summary.
    pub async fn generate_title(
        &self,
        summary: &str,
        transcript: &str,
        language: &str,
    ) -> Result<String, String> {
        let prompt = prompts::build_title_prompt(summary, transcript, language);
        self.chat_no_stream(&prompt).await
    }

    /// Query a transcript with streaming response.
    pub async fn query_transcript_streaming(
        &self,
        transcript: &str,
        question: &str,
        language: &str,
        app: tauri::AppHandle,
    ) -> Result<String, String> {
        let prompt = prompts::build_query_prompt(transcript, question, language);
        self.chat_streaming(&prompt, "query-chunk", "query-complete", app)
            .await
    }

    /// Query across multiple transcripts with streaming response.
    pub async fn query_corpus_streaming(
        &self,
        corpus: &str,
        question: &str,
        language: &str,
        app: tauri::AppHandle,
    ) -> Result<String, String> {
        let prompt = prompts::build_corpus_query_prompt(corpus, question, language);
        self.chat_streaming(&prompt, "query-chunk", "query-complete", app)
            .await
    }

    /// Internal: streaming chat completion.
    async fn chat_streaming(
        &self,
        prompt: &str,
        chunk_event: &str,
        complete_event: &str,
        app: tauri::AppHandle,
    ) -> Result<String, String> {
        let url = format!("{}/api/chat", self.base_url);
        let body = ChatRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            stream: true,
        };

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Ollama chat failed ({}): {}", status, body));
        }

        let mut stream = resp.bytes_stream();
        let mut full_text = String::new();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete JSON lines
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim().to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                if let Ok(chat_chunk) = serde_json::from_str::<ChatChunk>(&line) {
                    if let Some(msg) = chat_chunk.message {
                        let content = msg.content;
                        if !content.is_empty() {
                            full_text.push_str(&content);
                            let _ = app.emit(chunk_event, &content);
                        }
                    }
                    if chat_chunk.done {
                        let _ = app.emit(complete_event, ());
                    }
                }
            }
        }

        if full_text.is_empty() {
            return Err("Ollama returned an empty response. The model may not be loaded or the prompt was too long.".into());
        }

        Ok(full_text)
    }

    /// Internal: non-streaming chat completion.
    async fn chat_no_stream(&self, prompt: &str) -> Result<String, String> {
        let url = format!("{}/api/chat", self.base_url);
        let body = ChatRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            stream: false,
        };

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Ollama chat failed ({}): {}", status, body));
        }

        let chat_resp: ChatChunk = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        match chat_resp.message {
            Some(msg) => Ok(msg.content.trim().to_string()),
            None => Err("Ollama returned no message.".into()),
        }
    }
}