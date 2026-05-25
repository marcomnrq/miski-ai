use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::models::*;

/// Internal config file structure (settings + folders)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigData {
    #[serde(default)]
    settings: Settings,
    #[serde(default)]
    folders: Vec<Folder>,
}

impl Default for ConfigData {
    fn default() -> Self {
        Self {
            settings: Settings {
                language: "auto".into(),
                whisper_model: "small".into(),
                ollama_model: "llama3.2:3b".into(),
                ollama_url: "http://localhost:11434".into(),
                ai_provider: "local".into(),
                remote_ollama_url: String::new(),
                keep_recordings: false,
                notifications_enabled: true,
                user_name: None,
            },
            folders: Vec::new(),
        }
    }
}

/// Internal chat session file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatSessionData {
    session: ChatSession,
    messages: Vec<ChatMessage>,
}

/// JSON file-based persistence layer.
///
/// Directory layout:
///   ~/Library/Application Support/miski-ai/
///   ├── config.json              (settings + folders)
///   ├── meetings/
///   │   └── {uuid}.json          (one file per meeting)
///   ├── chat/
///   │   └── {uuid}.json          (one file per chat session + messages)
///   ├── recordings/              (temporary WAV files)
///   └── whisper_models/          (ggml-*.bin model weights)
pub struct JsonStore {
    base_dir: PathBuf,
    meetings_dir: PathBuf,
    chat_dir: PathBuf,
    recordings_dir: PathBuf,
    whisper_models_dir: PathBuf,
    config_path: PathBuf,
}

impl JsonStore {
    pub fn new() -> Result<Self> {
        let base_dir = dirs::data_dir()
            .context("Could not resolve application data directory")?
            .join("miski-ai");

        let store = Self {
            meetings_dir: base_dir.join("meetings"),
            chat_dir: base_dir.join("chat"),
            recordings_dir: base_dir.join("recordings"),
            whisper_models_dir: base_dir.join("whisper_models"),
            config_path: base_dir.join("config.json"),
            base_dir,
        };

        // Ensure all directories exist
        fs::create_dir_all(&store.base_dir)?;
        fs::create_dir_all(&store.meetings_dir)?;
        fs::create_dir_all(&store.chat_dir)?;
        fs::create_dir_all(&store.recordings_dir)?;
        fs::create_dir_all(&store.whisper_models_dir)?;

        // Initialize config if it doesn't exist
        if !store.config_path.exists() {
            let default_config = ConfigData::default();
            store.save_config_data(&default_config)?;
        }

        Ok(store)
    }

    // ── Config helpers ──────────────────────────────────────────

    fn load_config_data(&self) -> Result<ConfigData> {
        if !self.config_path.exists() {
            return Ok(ConfigData::default());
        }
        let content = fs::read_to_string(&self.config_path)
            .context("Failed to read config.json")?;
        let config: ConfigData = serde_json::from_str(&content)
            .context("Failed to parse config.json")?;
        Ok(config)
    }

    fn save_config_data(&self, config: &ConfigData) -> Result<()> {
        let json = serde_json::to_string_pretty(config)
            .context("Failed to serialize config")?;
        fs::write(&self.config_path, json)
            .context("Failed to write config.json")?;
        Ok(())
    }

    // ── Settings ────────────────────────────────────────────────

    pub fn get_settings(&self) -> Result<Settings> {
        let config = self.load_config_data()?;
        Ok(config.settings)
    }

    pub fn update_settings(&self, patch: &SettingsPatch) -> Result<()> {
        let mut config = self.load_config_data()?;

        if let Some(ref v) = patch.language { config.settings.language = v.clone(); }
        if let Some(ref v) = patch.whisper_model { config.settings.whisper_model = v.clone(); }
        if let Some(ref v) = patch.ollama_model { config.settings.ollama_model = v.clone(); }
        if let Some(ref v) = patch.ollama_url { config.settings.ollama_url = v.clone(); }
        if let Some(ref v) = patch.ai_provider { config.settings.ai_provider = v.clone(); }
        if let Some(ref v) = patch.remote_ollama_url { config.settings.remote_ollama_url = v.clone(); }
        if let Some(v) = patch.keep_recordings { config.settings.keep_recordings = v; }
        if let Some(v) = patch.notifications_enabled { config.settings.notifications_enabled = v; }
        if let Some(ref v) = patch.user_name { config.settings.user_name = Some(v.clone()); }

        self.save_config_data(&config)?;
        Ok(())
    }

    // ── Folders ─────────────────────────────────────────────────

    pub fn list_folders(&self) -> Result<Vec<Folder>> {
        let config = self.load_config_data()?;
        Ok(config.folders)
    }

    pub fn save_folders(&self, folders: &[Folder]) -> Result<()> {
        let mut config = self.load_config_data()?;
        config.folders = folders.to_vec();
        self.save_config_data(&config)?;
        Ok(())
    }

    // ── Meetings ────────────────────────────────────────────────

    pub fn list_meetings(&self) -> Result<Vec<MeetingSummary>> {
        let mut summaries = Vec::new();

        let entries = match fs::read_dir(&self.meetings_dir) {
            Ok(e) => e,
            Err(_) => return Ok(summaries),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(meeting) = self.load_meeting_file(&path) {
                let preview = meeting.summary_markdown.as_ref()
                    .map(|s| {
                        // Take first 200 chars, strip markdown headers
                        let stripped = s.lines()
                            .filter(|l| !l.starts_with('#'))
                            .collect::<Vec<_>>()
                            .join(" ")
                            .trim()
                            .to_string();
                        if stripped.len() > 200 {
                            format!("{}...", &stripped[..200])
                        } else {
                            stripped
                        }
                    });

                summaries.push(MeetingSummary {
                    id: meeting.id,
                    title: meeting.title,
                    created_at: meeting.created_at,
                    duration_seconds: meeting.duration_seconds,
                    language: meeting.language,
                    is_diarised: meeting.is_diarised,
                    folder_id: meeting.folder_id,
                    summary_preview: preview,
                });
            }
        }

        // Sort by created_at descending (newest first)
        summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(summaries)
    }

    pub fn get_meeting(&self, id: &str) -> Result<Meeting> {
        let path = self.meetings_dir.join(format!("{}.json", id));
        if !path.exists() {
            anyhow::bail!("Meeting not found: {}", id);
        }
        self.load_meeting_file(&path)
    }

    pub fn save_meeting(&self, meeting: &Meeting) -> Result<()> {
        let path = self.meetings_dir.join(format!("{}.json", meeting.id));
        let json = serde_json::to_string_pretty(meeting)
            .context("Failed to serialize meeting")?;
        fs::write(&path, json)
            .context("Failed to write meeting file")?;
        Ok(())
    }

    pub fn delete_meeting(&self, id: &str) -> Result<()> {
        let path = self.meetings_dir.join(format!("{}.json", id));
        if path.exists() {
            fs::remove_file(&path)?;
        }
        // Also try to delete associated audio file
        let meeting = self.load_meeting_file(&path).ok();
        if let Some(m) = meeting {
            if let Some(ref audio_path) = m.audio_path {
                let _ = fs::remove_file(audio_path);
            }
        }
        Ok(())
    }

    fn load_meeting_file(&self, path: &Path) -> Result<Meeting> {
        let content = fs::read_to_string(path)
            .context("Failed to read meeting file")?;
        serde_json::from_str(&content)
            .context("Failed to parse meeting JSON")
    }

    // ── Chat Sessions ───────────────────────────────────────────

    pub fn list_chat_sessions(&self) -> Result<Vec<ChatSession>> {
        let mut sessions = Vec::new();

        let entries = match fs::read_dir(&self.chat_dir) {
            Ok(e) => e,
            Err(_) => return Ok(sessions),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(data) = self.load_chat_file(&path) {
                sessions.push(data.session);
            }
        }

        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(sessions)
    }

    pub fn get_chat_session(&self, id: &str) -> Result<(ChatSession, Vec<ChatMessage>)> {
        let path = self.chat_dir.join(format!("{}.json", id));
        if !path.exists() {
            anyhow::bail!("Chat session not found: {}", id);
        }
        let data = self.load_chat_file(&path)?;
        Ok((data.session, data.messages))
    }

    pub fn save_chat_session(&self, session: &ChatSession, messages: &[ChatMessage]) -> Result<()> {
        let path = self.chat_dir.join(format!("{}.json", session.id));
        let data = ChatSessionData {
            session: session.clone(),
            messages: messages.to_vec(),
        };
        let json = serde_json::to_string_pretty(&data)?;
        fs::write(&path, json)?;
        Ok(())
    }

    pub fn delete_chat_session(&self, id: &str) -> Result<()> {
        let path = self.chat_dir.join(format!("{}.json", id));
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    fn load_chat_file(&self, path: &Path) -> Result<ChatSessionData> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).context("Failed to parse chat session JSON")
    }

    // ── Path accessors ──────────────────────────────────────────

    pub fn recordings_dir(&self) -> &PathBuf {
        &self.recordings_dir
    }

    pub fn whisper_models_dir(&self) -> &PathBuf {
        &self.whisper_models_dir
    }

    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }
}