mod models;

use models::*;
use tauri::State;

// ── Managed State ──────────────────────────────────────────────

pub struct AppState {
    // TODO: AudioRecorder when cpal is integrated
    // TODO: WhisperTranscriber when whisper-rs is integrated
    // TODO: OllamaClient when reqwest is integrated
}

pub struct Database {
    // TODO: rusqlite::Connection when rusqlite is integrated
}

// ── Recording Commands ─────────────────────────────────────────

#[tauri::command]
async fn start_recording(
    name: String,
    state: State<'_, AppState>,
) -> Result<RecordingInfo, String> {
    // TODO: Implement with cpal audio capture
    Err("Not yet implemented".into())
}

#[tauri::command]
async fn stop_recording(state: State<'_, AppState>) -> Result<String, String> {
    // TODO: Stop cpal stream, save WAV via hound
    Err("Not yet implemented".into())
}

#[tauri::command]
async fn pause_recording(state: State<'_, AppState>) -> Result<(), String> {
    // TODO: Pause audio stream
    Err("Not yet implemented".into())
}

#[tauri::command]
async fn resume_recording(state: State<'_, AppState>) -> Result<(), String> {
    // TODO: Resume audio stream
    Err("Not yet implemented".into())
}

#[tauri::command]
async fn get_audio_level(state: State<'_, AppState>) -> Result<AudioLevel, String> {
    // TODO: Return RMS level from audio buffer
    Ok(AudioLevel { rms: 0.0, peak: 0.0, duration_secs: 0.0 })
}

// ── Meeting Commands ───────────────────────────────────────────

#[tauri::command]
async fn list_meetings(db: State<'_, Database>) -> Result<Vec<MeetingSummary>, String> {
    // TODO: Query SQLite meetings table
    Ok(vec![])
}

#[tauri::command]
async fn get_meeting(id: String, db: State<'_, Database>) -> Result<Meeting, String> {
    // TODO: Query SQLite for single meeting
    Err("Not yet implemented".into())
}

#[tauri::command]
async fn delete_meeting(id: String, db: State<'_, Database>) -> Result<(), String> {
    // TODO: Delete from SQLite
    Err("Not yet implemented".into())
}

#[tauri::command]
async fn rename_meeting(id: String, name: String, db: State<'_, Database>) -> Result<(), String> {
    // TODO: Update title in SQLite
    Err("Not yet implemented".into())
}

#[tauri::command]
async fn save_notes(id: String, notes: String, db: State<'_, Database>) -> Result<(), String> {
    // TODO: Update notes in SQLite
    Err("Not yet implemented".into())
}

#[tauri::command]
async fn export_meeting_markdown(id: String, db: State<'_, Database>) -> Result<String, String> {
    // TODO: Generate markdown export
    Err("Not yet implemented".into())
}

// ── AI / Streaming Commands ────────────────────────────────────

#[tauri::command]
async fn query_meeting(
    id: String,
    question: String,
    app: tauri::AppHandle,
    db: State<'_, Database>,
) -> Result<(), String> {
    // TODO: Load meeting, stream Q&A via Ollama, emit query-chunk events
    Err("Not yet implemented".into())
}

#[tauri::command]
async fn query_all(
    question: String,
    folder_id: Option<String>,
    app: tauri::AppHandle,
    db: State<'_, Database>,
) -> Result<(), String> {
    // TODO: Load corpus, stream Q&A via Ollama, emit query-chunk events
    Err("Not yet implemented".into())
}

// ── Chat Commands ──────────────────────────────────────────────

#[tauri::command]
async fn list_chat_sessions(db: State<'_, Database>) -> Result<Vec<ChatSession>, String> {
    // TODO: Query SQLite chat_sessions table
    Ok(vec![])
}

#[tauri::command]
async fn create_chat_session(
    scope: String,
    folder_id: Option<String>,
    db: State<'_, Database>,
) -> Result<ChatSession, String> {
    // TODO: Insert into SQLite
    Err("Not yet implemented".into())
}

#[tauri::command]
async fn delete_chat_session(id: String, db: State<'_, Database>) -> Result<(), String> {
    // TODO: Delete from SQLite (cascades messages)
    Err("Not yet implemented".into())
}

// ── Settings Commands ──────────────────────────────────────────

#[tauri::command]
async fn get_settings(db: State<'_, Database>) -> Result<Settings, String> {
    // TODO: Read from SQLite settings table
    Ok(Settings {
        language: "auto".into(),
        whisper_model: "small".into(),
        ollama_model: "llama3.2:3b".into(),
        ollama_url: "http://localhost:11434".into(),
        ai_provider: "local".into(),
        remote_ollama_url: String::new(),
        keep_recordings: false,
        notifications_enabled: true,
        user_name: None,
    })
}

#[tauri::command]
async fn update_settings(
    settings: SettingsPatch,
    db: State<'_, Database>,
) -> Result<(), String> {
    // TODO: Patch SQLite settings table
    Err("Not yet implemented".into())
}

// ── Setup / Model Commands ─────────────────────────────────────

#[tauri::command]
async fn check_setup(state: State<'_, AppState>) -> Result<SetupStatus, String> {
    // TODO: Check Ollama, whisper model, mic availability
    Ok(SetupStatus {
        ollama_installed: false,
        ollama_running: false,
        whisper_model_downloaded: false,
        ollama_model_pulled: false,
        microphone_available: false,
    })
}

#[tauri::command]
async fn list_ollama_models(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    // TODO: HTTP GET /api/tags to Ollama
    Ok(vec![])
}

#[tauri::command]
async fn pull_ollama_model(
    name: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // TODO: Stream POST /api/pull, emit model-pull-progress events
    Err("Not yet implemented".into())
}

#[tauri::command]
async fn download_whisper_model(
    model_size: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // TODO: Download ggml-{size}.bin from HuggingFace
    Err("Not yet implemented".into())
}

// ── Folder Commands ────────────────────────────────────────────

#[tauri::command]
async fn list_folders(db: State<'_, Database>) -> Result<Vec<Folder>, String> {
    // TODO: Query SQLite folders table
    Ok(vec![])
}

#[tauri::command]
async fn create_folder(
    name: String,
    icon: String,
    color: String,
    db: State<'_, Database>,
) -> Result<Folder, String> {
    // TODO: Insert into SQLite
    Err("Not yet implemented".into())
}

#[tauri::command]
async fn update_folder(
    id: String,
    name: String,
    icon: String,
    color: String,
    db: State<'_, Database>,
) -> Result<(), String> {
    // TODO: Update SQLite
    Err("Not yet implemented".into())
}

#[tauri::command]
async fn delete_folder(id: String, db: State<'_, Database>) -> Result<(), String> {
    // TODO: Delete from SQLite
    Err("Not yet implemented".into())
}

// ── App Entry Point ────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {})
        .manage(Database {})
        .invoke_handler(tauri::generate_handler![
            // Recording
            start_recording,
            stop_recording,
            pause_recording,
            resume_recording,
            get_audio_level,
            // Meetings
            list_meetings,
            get_meeting,
            delete_meeting,
            rename_meeting,
            save_notes,
            export_meeting_markdown,
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
