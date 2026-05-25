mod ai;
mod audio;
mod models;
mod pipeline;
mod storage;
mod transcription;

use models::*;
use storage::JsonStore;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};
use cpal::traits::HostTrait;

use audio::AudioRecorderHandle;
use ai::OllamaClient;

// ── Managed State ──────────────────────────────────────────────

/// Holds the audio recorder when a recording is active.
pub struct AppState {
    /// Active audio recorder (Some while recording, None otherwise)
    pub recorder: Mutex<Option<AudioRecorderHandle>>,
    /// Store is shared so async tasks can access it
    pub store: Arc<JsonStore>,
    /// Recording metadata while recording is active
    pub recording_meta: Mutex<Option<RecordingMeta>>,
}

/// Metadata for the current recording session
struct RecordingMeta {
    pub id: String,
    pub session_name: String,
    pub notes: String,
    pub started_at: String,
}

// ── Recording Commands ─────────────────────────────────────────

#[tauri::command]
async fn start_recording(
    name: String,
    state: State<'_, AppState>,
) -> Result<RecordingInfo, String> {
    let mut recorder_guard = state.recorder.lock().map_err(|e| e.to_string())?;
    if recorder_guard.is_some() {
        return Err("Already recording".into());
    }

    // Create and start the real audio recorder (spawns dedicated audio thread)
    let recorder = AudioRecorderHandle::start()?;

    let id = uuid::Uuid::new_v4().to_string();
    let started_at = chrono::Utc::now().to_rfc3339();
    let wav_path = state.store.recordings_dir().join(format!("{}.wav", id));

    // Store metadata
    let mut meta = state.recording_meta.lock().map_err(|e| e.to_string())?;
    *meta = Some(RecordingMeta {
        id: id.clone(),
        session_name: name.clone(),
        notes: String::new(),
        started_at: started_at.clone(),
    });

    *recorder_guard = Some(recorder);

    Ok(RecordingInfo {
        id,
        path: wav_path.to_string_lossy().to_string(),
        session_name: name,
        started_at,
    })
}

#[tauri::command]
async fn stop_recording(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Take the recorder out (stops recording and saves WAV)
    // Take the recorder out
    let recorder_guard = state.recorder.lock().map_err(|e| e.to_string())?;
    let recorder = recorder_guard
        .as_ref()
        .ok_or("Not recording")?;

    // Take metadata
    let mut meta_guard = state.recording_meta.lock().map_err(|e| e.to_string())?;
    let meta = meta_guard
        .take()
        .ok_or("No recording metadata")?;

    let wav_path = state.store.recordings_dir().join(format!("{}.wav", meta.id));

    // Stop the recorder — saves WAV file (blocks until audio thread finishes)
    let _duration = recorder.stop(&wav_path)?;

    // Now remove the recorder from state
    drop(recorder_guard);
    state.recorder.lock().map_err(|e| e.to_string())?.take();

    // Emit processing-started event
    let _ = app.emit("processing-status", "Transcribing audio...");

    // Clone what we need for the async pipeline task
    let store = state.store.clone();
    let wav_path_clone = wav_path.clone();
    let session_name = meta.session_name.clone();
    let notes = if meta.notes.is_empty() {
        None
    } else {
        Some(meta.notes)
    };

    // Spawn the processing pipeline in a background task
    tauri::async_runtime::spawn(async move {
        let _ = pipeline::process_recording(
            &wav_path_clone,
            &session_name,
            notes.as_deref(),
            app,
            &store,
        )
        .await
        .map_err(|e| {
            eprintln!("Processing pipeline error: {}", e);
            e
        });
    });

    Ok(wav_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn pause_recording(state: State<'_, AppState>) -> Result<(), String> {
    let recorder_guard = state.recorder.lock().map_err(|e| e.to_string())?;
    match recorder_guard.as_ref() {
        Some(recorder) => recorder.pause(),
        None => Err("Not recording".into()),
    }
}

#[tauri::command]
async fn resume_recording(state: State<'_, AppState>) -> Result<(), String> {
    let recorder_guard = state.recorder.lock().map_err(|e| e.to_string())?;
    match recorder_guard.as_ref() {
        Some(recorder) => recorder.resume(),
        None => Err("Not recording".into()),
    }
}

#[tauri::command]
async fn get_audio_level(state: State<'_, AppState>) -> Result<AudioLevel, String> {
    let recorder_guard = state.recorder.lock().map_err(|e| e.to_string())?;
    match recorder_guard.as_ref() {
        Some(recorder) => Ok(AudioLevel {
            rms: recorder.get_audio_level(),
            peak: 0.0,
            duration_secs: recorder.get_duration(),
        }),
        None => Ok(AudioLevel {
            rms: 0.0,
            peak: 0.0,
            duration_secs: 0.0,
        }),
    }
}

#[tauri::command]
async fn update_recording_notes(
    notes: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut meta_guard = state.recording_meta.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut meta) = *meta_guard {
        meta.notes = notes;
        Ok(())
    } else {
        Err("Not recording".into())
    }
}

// ── Meeting Commands ───────────────────────────────────────────

#[tauri::command]
async fn list_meetings(state: State<'_, AppState>) -> Result<Vec<MeetingSummary>, String> {
    state.store.list_meetings().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_meeting(id: String, state: State<'_, AppState>) -> Result<Meeting, String> {
    state.store.get_meeting(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_meeting(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.store.delete_meeting(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn rename_meeting(
    id: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut meeting = state.store.get_meeting(&id).map_err(|e| e.to_string())?;
    meeting.title = name;
    meeting.updated_at = chrono::Utc::now().to_rfc3339();
    state.store.save_meeting(&meeting).map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_notes(
    id: String,
    notes: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut meeting = state.store.get_meeting(&id).map_err(|e| e.to_string())?;
    meeting.notes = Some(notes);
    meeting.updated_at = chrono::Utc::now().to_rfc3339();
    state.store.save_meeting(&meeting).map_err(|e| e.to_string())
}

#[tauri::command]
async fn export_meeting_md(
    id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let meeting = state.store.get_meeting(&id).map_err(|e| e.to_string())?;
    Ok(storage::markdown::export_meeting_markdown(&meeting))
}

// ── AI / Streaming Commands ────────────────────────────────────

#[tauri::command]
async fn query_meeting(
    id: String,
    question: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let meeting = state.store.get_meeting(&id).map_err(|e| e.to_string())?;
    let transcript = meeting
        .transcript_text
        .ok_or("Meeting has no transcript")?;
    let settings = state.store.get_settings().map_err(|e| e.to_string())?;

    let client = OllamaClient::new(&settings.ollama_url, &settings.ollama_model);
    client
        .query_transcript_streaming(&transcript, &question, &settings.language, app)
        .await?;

    Ok(())
}

#[tauri::command]
async fn query_all(
    question: String,
    folder_id: Option<String>,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let meetings = state.store.list_meetings().map_err(|e| e.to_string())?;
    let settings = state.store.get_settings().map_err(|e| e.to_string())?;

    // Build corpus from all meetings (or filtered by folder)
    let mut corpus_parts = Vec::new();
    for summary in &meetings {
        if let Some(ref fid) = folder_id {
            if summary.folder_id.as_deref() != Some(fid.as_str()) {
                continue;
            }
        }
        if let Ok(meeting) = state.store.get_meeting(&summary.id) {
            if let Some(ref transcript) = meeting.transcript_text {
                corpus_parts.push(format!(
                    "--- Meeting: {} ({}) ---\n{}",
                    meeting.title, meeting.created_at, transcript
                ));
            }
        }
    }

    if corpus_parts.is_empty() {
        return Err("No transcripts found to query.".into());
    }

    let corpus = corpus_parts.join("\n\n");
    let client = OllamaClient::new(&settings.ollama_url, &settings.ollama_model);
    client
        .query_corpus_streaming(&corpus, &question, &settings.language, app)
        .await?;

    Ok(())
}

// ── Chat Commands ──────────────────────────────────────────────

#[tauri::command]
async fn list_chat_sessions(state: State<'_, AppState>) -> Result<Vec<ChatSession>, String> {
    state.store.list_chat_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_chat_session(
    scope: String,
    folder_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<ChatSession, String> {
    let session = ChatSession {
        id: uuid::Uuid::new_v4().to_string(),
        title: None,
        scope,
        folder_id,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    state
        .store
        .save_chat_session(&session, &[])
        .map_err(|e| e.to_string())?;
    Ok(session)
}

#[tauri::command]
async fn delete_chat_session(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.store.delete_chat_session(&id).map_err(|e| e.to_string())
}

// ── Settings Commands ──────────────────────────────────────────

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    state.store.get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_settings(
    settings: SettingsPatch,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .store
        .update_settings(&settings)
        .map_err(|e| e.to_string())
}

// ── Setup / Model Commands ─────────────────────────────────────

#[tauri::command]
async fn check_setup(state: State<'_, AppState>) -> Result<SetupStatus, String> {
    let settings = state.store.get_settings().map_err(|e| e.to_string())?;

    // Check Ollama
    let client = OllamaClient::new(&settings.ollama_url, &settings.ollama_model);
    let ollama_running = client.health_check().await.unwrap_or(false);

    // Check whisper model
    let model_path = state
        .store
        .whisper_models_dir()
        .join(format!("ggml-{}.bin", settings.whisper_model));
    let whisper_downloaded = model_path.exists();

    // Check Ollama model
    let ollama_model_pulled = if ollama_running {
        client
            .list_models()
            .await
            .map(|models| {
                models
                    .iter()
                    .any(|m| m.name == settings.ollama_model || m.name.starts_with(&settings.ollama_model))
            })
            .unwrap_or(false)
    } else {
        false
    };

    // Check microphone
    let mic_available = cpal::default_host().default_input_device().is_some();

    Ok(SetupStatus {
        ollama_installed: ollama_running, // If it responds, it's installed
        ollama_running,
        whisper_model_downloaded: whisper_downloaded,
        ollama_model_pulled,
        microphone_available: mic_available,
    })
}

#[tauri::command]
async fn list_ollama_models(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    let settings = state.store.get_settings().map_err(|e| e.to_string())?;
    let client = OllamaClient::new(&settings.ollama_url, &settings.ollama_model);
    client.list_models().await
}

#[tauri::command]
async fn pull_ollama_model(
    name: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let settings = state.store.get_settings().map_err(|e| e.to_string())?;
    let client = OllamaClient::new(&settings.ollama_url, &settings.ollama_model);
    client.pull_model(&name, app).await
}

#[tauri::command]
async fn download_whisper_model(
    model_size: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Validate model size
    let valid_sizes = [
        "tiny",
        "base",
        "small",
        "medium",
        "large",
        "large-v3-turbo",
    ];
    if !valid_sizes.contains(&model_size.as_str()) {
        return Err(format!(
            "Invalid whisper model size: {}. Must be one of: {}",
            model_size,
            valid_sizes.join(", ")
        ));
    }

    let model_dir = state.store.whisper_models_dir();
    let model_path = model_dir.join(format!("ggml-{}.bin", model_size));

    if model_path.exists() {
        return Ok(()); // Already downloaded
    }

    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
        model_size
    );

    // Download with progress
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to start download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download whisper model: HTTP {}",
            response.status()
        ));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut file =
        std::fs::File::create(&model_path).map_err(|e| format!("Failed to create file: {}", e))?;

    use futures_util::StreamExt;
    use std::io::Write;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk).map_err(|e| format!("Write error: {}", e))?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let percent = ((downloaded as f64 / total_size as f64) * 100.0) as u32;
            let _ = app.emit("whisper-download-progress", percent);
        }
    }

    let _ = app.emit("whisper-download-complete", ());
    Ok(())
}

// ── Folder Commands ────────────────────────────────────────────

#[tauri::command]
async fn list_folders(state: State<'_, AppState>) -> Result<Vec<Folder>, String> {
    state.store.list_folders().map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_folder(
    name: String,
    icon: String,
    color: String,
    state: State<'_, AppState>,
) -> Result<Folder, String> {
    let folder = Folder {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        icon,
        color,
        sort_order: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let mut folders = state.store.list_folders().map_err(|e| e.to_string())?;
    folders.push(folder.clone());
    state.store.save_folders(&folders).map_err(|e| e.to_string())?;
    Ok(folder)
}

#[tauri::command]
async fn update_folder(
    id: String,
    name: String,
    icon: String,
    color: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut folders = state.store.list_folders().map_err(|e| e.to_string())?;
    let folder = folders
        .iter_mut()
        .find(|f| f.id == id)
        .ok_or_else(|| format!("Folder not found: {}", id))?;
    folder.name = name;
    folder.icon = icon;
    folder.color = color;
    state.store.save_folders(&folders).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_folder(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut folders = state.store.list_folders().map_err(|e| e.to_string())?;
    folders.retain(|f| f.id != id);
    state.store.save_folders(&folders).map_err(|e| e.to_string())
}

// ── App Entry Point ────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let store = JsonStore::new().expect("Failed to initialize storage");
    let store = Arc::new(store);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            recorder: Mutex::new(None),
            store: store.clone(),
            recording_meta: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            // Recording
            start_recording,
            stop_recording,
            pause_recording,
            resume_recording,
            get_audio_level,
            update_recording_notes,
            // Meetings
            list_meetings,
            get_meeting,
            delete_meeting,
            rename_meeting,
            save_notes,
            export_meeting_md,
            // AI / Streaming
            query_meeting,
            query_all,
            // Chat
            list_chat_sessions,
            create_chat_session,
            delete_chat_session,
            // Settings
            get_settings,
            update_settings,
            // Setup / Models
            check_setup,
            list_ollama_models,
            pull_ollama_model,
            download_whisper_model,
            // Folders
            list_folders,
            create_folder,
            update_folder,
            delete_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}