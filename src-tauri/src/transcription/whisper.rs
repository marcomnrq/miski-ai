use std::path::Path;

use crate::models::{Segment, TranscriptResult};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Whisper-based audio transcriber.
///
/// Loads a ggml model file and transcribes WAV audio files into text with
/// per-segment timing information.
pub struct WhisperTranscriber {
    ctx: WhisperContext,
    model_size: String,
}

impl WhisperTranscriber {
    /// Load a whisper model from the given path.
    ///
    /// `model_path` should point to a `ggml-{size}.bin` file.
    pub fn new(model_path: &Path) -> Result<Self, String> {
        if !model_path.exists() {
            return Err(format!(
                "Whisper model not found at: {}. Please download it first via Settings.",
                model_path.display()
            ));
        }

        let params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(&model_path.to_string_lossy(), params)
            .map_err(|e| format!("Failed to load whisper model: {}", e))?;

        Ok(Self {
            ctx,
            model_size: model_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .replace("ggml-", ""),
        })
    }

    /// Transcribe an audio file.
    ///
    /// `audio_path` must be a 16-bit PCM WAV file at 16 kHz mono.
    /// `language` can be None for auto-detection, or a language code like "en", "es", etc.
    pub fn transcribe(
        &self,
        audio_path: &Path,
        language: Option<&str>,
    ) -> Result<TranscriptResult, String> {
        // 1. Read WAV file
        let mut reader = hound::WavReader::open(audio_path)
            .map_err(|e| format!("Failed to open WAV file: {}", e))?;

        let spec = reader.spec();
        if spec.sample_rate != 16_000 {
            return Err(format!(
                "WAV file must be 16000 Hz, got {} Hz. Recording pipeline may be broken.",
                spec.sample_rate
            ));
        }
        if spec.channels != 1 {
            return Err(format!(
                "WAV file must be mono (1 channel), got {} channels.",
                spec.channels
            ));
        }

        let i16_samples: Vec<i16> = reader
            .samples()
            .map(|s| s.unwrap_or(0i16))
            .collect();

        if i16_samples.is_empty() {
            return Err("WAV file contains no audio samples.".into());
        }

        let duration_seconds = i16_samples.len() as f64 / 16_000.0;

        // 2. Convert i16 samples to f32 for whisper
        let mut f32_interleaved = Vec::with_capacity(i16_samples.len());
        whisper_rs::convert_integer_to_float_audio(&i16_samples, &mut f32_interleaved)
            .map_err(|e| format!("Failed to convert audio samples: {}", e))?;

        // Audio is already mono from our recorder
        let audio_data = f32_interleaved;

        // 3. Create whisper state
        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| format!("Failed to create whisper state: {}", e))?;

        // 4. Build whisper parameters
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Use multiple threads for faster transcription
        params.set_n_threads(4);

        // Suppress verbose output
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // We need segments for diarisation
        params.set_single_segment(false);

        // Get accurate timestamps for segment boundaries
        params.set_token_timestamps(true);

        // Set language if specified
        if let Some(lang) = language {
            params.set_language(Some(lang));
        }

        // 5. Run transcription
        state
            .full(params, &audio_data)
            .map_err(|e| format!("Whisper transcription failed: {}", e))?;

        // 6. Extract segments
        let n_segments = state
            .full_n_segments()
            .map_err(|e| format!("Failed to get segment count: {}", e))?;

        let mut segments = Vec::with_capacity(n_segments as usize);
        let mut full_text_parts = Vec::with_capacity(n_segments as usize);

        for i in 0..n_segments {
            let text = state
                .full_get_segment_text(i)
                .map_err(|e| format!("Failed to get segment text: {}", e))?
                .trim()
                .to_string();

            // Timestamps are in centiseconds (10ms units), convert to milliseconds
            let start_ms = state
                .full_get_segment_t0(i)
                .map_err(|e| format!("Failed to get segment start: {}", e))?
                as i64
                * 10;
            let end_ms = state
                .full_get_segment_t1(i)
                .map_err(|e| format!("Failed to get segment end: {}", e))?
                as i64
                * 10;

            if !text.is_empty() {
                full_text_parts.push(text.clone());
                segments.push(Segment {
                    start_ms,
                    end_ms,
                    text,
                });
            }
        }

        let full_text = full_text_parts.join(" ");

        // Language detection: use provided language or None for auto
        let detected_language = language.map(|s| s.to_string());

        Ok(TranscriptResult {
            text: full_text,
            segments,
            detected_language,
            duration_seconds,
        })
    }

    /// Get the model size name (e.g., "small", "base", etc.)
    pub fn model_size(&self) -> &str {
        &self.model_size
    }
}