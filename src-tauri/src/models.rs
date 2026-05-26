use serde::{Deserialize, Serialize};

/// Recording info returned when starting a recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingInfo {
    pub id: String,
    pub path: String,
    pub session_name: String,
    pub started_at: String,
}

/// Audio level for UI waveform visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioLevel {
    pub rms: f32,
    pub peak: f32,
    pub duration_secs: f64,
}

/// A single whisper transcription segment with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

/// Full transcription result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptResult {
    pub text: String,
    pub segments: Vec<Segment>,
    pub detected_language: Option<String>,
    pub duration_seconds: f64,
}

/// A segment labeled with a speaker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelledSegment {
    pub speaker: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

/// Summary of a meeting for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingSummary {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub duration_seconds: Option<f64>,
    pub language: String,
    pub is_diarised: bool,
    pub folder_id: Option<String>,
    pub summary_preview: Option<String>,
}

/// Full meeting detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: String,
    pub title: String,
    pub audio_path: Option<String>,
    pub transcript_text: Option<String>,
    pub diarised_text: Option<String>,
    pub summary_markdown: Option<String>,
    pub notes: Option<String>,
    pub language: String,
    pub is_diarised: bool,
    pub duration_seconds: Option<f64>,
    pub whisper_model: Option<String>,
    pub ollama_model: Option<String>,
    pub folder_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Chat session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: Option<String>,
    pub scope: String,
    pub folder_id: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub language: String,
    pub whisper_model: String,
    pub ollama_model: String,
    pub ollama_url: String,
    pub ai_provider: String,
    pub remote_ollama_url: String,
    pub keep_recordings: bool,
    pub notifications_enabled: bool,
    pub user_name: Option<String>,
}

/// Partial settings update
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsPatch {
    pub language: Option<String>,
    pub whisper_model: Option<String>,
    pub ollama_model: Option<String>,
    pub ollama_url: Option<String>,
    pub ai_provider: Option<String>,
    pub remote_ollama_url: Option<String>,
    pub keep_recordings: Option<bool>,
    pub notifications_enabled: Option<bool>,
    pub user_name: Option<String>,
}

/// Setup status for first-run checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStatus {
    pub ollama_installed: bool,
    pub ollama_running: bool,
    pub whisper_model_downloaded: bool,
    pub ollama_model_pulled: bool,
    pub microphone_available: bool,
}

/// Ollama model info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: String,
    pub description: String,
    pub installed: bool,
}

/// Model pull progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullProgress {
    pub status: String,
    pub percent: u32,
}

/// Folder for organizing meetings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub sort_order: i32,
    pub created_at: String,
}
