use std::path::Path;

use tauri::Emitter;

use crate::ai::OllamaClient;
use crate::storage::JsonStore;
use crate::transcription::{Diariser, WhisperTranscriber};
use crate::models::Meeting;

/// Full recording processing pipeline:
/// WAV → Transcribe → Diarise → Summarize → Generate Title → Save Meeting
pub async fn process_recording(
    wav_path: &Path,
    session_name: &str,
    notes: Option<&str>,
    app: tauri::AppHandle,
    store: &JsonStore,
) -> Result<Meeting, String> {
    // 1. Load settings
    let settings = store.get_settings().map_err(|e| e.to_string())?;

    // 2. Load whisper model
    let model_path = store
        .whisper_models_dir()
        .join(format!("ggml-{}.bin", settings.whisper_model));

    let transcriber = WhisperTranscriber::new(&model_path)?;

    // 3. Transcribe
    let lang = if settings.language == "auto" {
        None
    } else {
        Some(settings.language.as_str())
    };

    let result = transcriber.transcribe(wav_path, lang)?;

    // 4. Diarise
    let diariser = Diariser::new();
    let labelled = diariser.detect_turns(&result.segments);

    let diarised_text = if labelled.len() > 1 {
        Some(
            labelled
                .iter()
                .map(|s| format!("[{}] {}", s.speaker, s.text))
                .collect::<Vec<_>>()
                .join("\n\n"),
        )
    } else {
        None
    };

    // 5. Summarize (streaming)
    let client = OllamaClient::new(&settings.ollama_url, &settings.ollama_model);
    let duration_minutes = (result.duration_seconds / 60.0).ceil() as u32;

    let summary = client
        .summarize_streaming(
            &result.text,
            duration_minutes,
            &settings.language,
            notes,
            app.clone(),
        )
        .await?;

    // 6. Generate title
    let title = client
        .generate_title(&summary, &result.text, &settings.language)
        .await
        .unwrap_or_else(|_| {
            // Fallback: use session name or first words of transcript
            if session_name.is_empty() {
                let words: Vec<&str> = result.text.split_whitespace().take(6).collect();
                words.join(" ")
            } else {
                session_name.to_string()
            }
        });

    // 7. Build meeting
    let meeting = Meeting {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        audio_path: if settings.keep_recordings {
            Some(wav_path.to_string_lossy().to_string())
        } else {
            None
        },
        transcript_text: Some(result.text),
        diarised_text,
        summary_markdown: Some(summary),
        notes: notes.map(String::from),
        language: result
            .detected_language
            .unwrap_or_else(|| settings.language.clone()),
        is_diarised: labelled.len() > 1,
        duration_seconds: Some(result.duration_seconds),
        whisper_model: Some(settings.whisper_model),
        ollama_model: Some(settings.ollama_model),
        folder_id: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    // 8. Save meeting
    store.save_meeting(&meeting).map_err(|e| e.to_string())?;

    // 9. Clean up WAV unless keep_recordings is set
    if !settings.keep_recordings {
        let _ = std::fs::remove_file(wav_path);
    }

    // 10. Emit processing-complete
    app.emit("processing-complete", &meeting.id)
        .map_err(|e| e.to_string())?;

    Ok(meeting)
}