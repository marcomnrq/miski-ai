use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Target sample rate for whisper (must be 16000 Hz).
const SAMPLE_RATE: u32 = 16_000;
/// Mono channel for whisper.
const CHANNELS: u16 = 1;
/// Number of recent samples to keep for audio level calculation (~100ms at 16kHz).
const RECENT_SAMPLES_COUNT: usize = 1600;

// ── Thread-safe handle ─────────────────────────────────────────

/// Thread-safe handle to an audio recording session.
///
/// The actual `cpal::Stream` lives in a dedicated thread (since cpal::Stream
/// is not Send on macOS). Communication happens via channels and atomics.
pub struct AudioRecorderHandle {
    is_paused: Arc<AtomicBool>,
    recent_samples: Arc<Mutex<Vec<f32>>>,
    sample_count: Arc<AtomicU64>,
    start_time: Instant,
    stop_sender: Mutex<Option<std::sync::mpsc::Sender<()>>>,
    result_receiver: Mutex<Option<std::sync::mpsc::Receiver<Result<Vec<f32>, String>>>>,
}

// Safety: The handle only contains Send-safe types. The cpal::Stream
// is owned by a dedicated thread and never crosses thread boundaries.
unsafe impl Send for AudioRecorderHandle {}
unsafe impl Sync for AudioRecorderHandle {}

impl AudioRecorderHandle {
    /// Start a new recording session.
    ///
    /// Spawns a dedicated thread that owns the cpal::Stream and collects
    /// audio samples into a buffer.
    pub fn start() -> Result<Self, String> {
        let is_paused = Arc::new(AtomicBool::new(false));
        let recent_samples = Arc::new(Mutex::new(Vec::with_capacity(RECENT_SAMPLES_COUNT * 2)));
        let sample_count = Arc::new(AtomicU64::new(0));

        let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
        let (result_tx, result_rx) = std::sync::mpsc::channel::<Result<Vec<f32>, String>>();

        let is_paused_clone = is_paused.clone();
        let recent_samples_clone = recent_samples.clone();
        let sample_count_clone = sample_count.clone();

        // Spawn a dedicated thread for audio capture
        std::thread::Builder::new()
            .name("miski-audio-capture".into())
            .spawn(move || {
                let result = run_audio_capture(
                    is_paused_clone,
                    recent_samples_clone,
                    sample_count_clone,
                    stop_rx,
                );
                let _ = result_tx.send(result);
            })
            .map_err(|e| format!("Failed to spawn audio thread: {}", e))?;

        Ok(Self {
            is_paused,
            recent_samples,
            sample_count,
            start_time: Instant::now(),
            stop_sender: Mutex::new(Some(stop_tx)),
            result_receiver: Mutex::new(Some(result_rx)),
        })
    }

    /// Stop recording and save the captured audio to a WAV file.
    ///
    /// Returns the duration of the recording in seconds.
    pub fn stop(&self, output_path: &Path) -> Result<f64, String> {
        // Send stop signal to the audio thread
        {
            let mut stop_guard = self.stop_sender.lock().map_err(|e| e.to_string())?;
            if let Some(tx) = stop_guard.take() {
                let _ = tx.send(());
            } else {
                return Err("Recording already stopped.".into());
            }
        }

        // Wait for the result from the audio thread
        let samples = {
            let mut result_guard = self.result_receiver.lock().map_err(|e| e.to_string())?;
            let receiver = result_guard.take().ok_or("No result receiver")?;
            receiver
                .recv()
                .map_err(|_| "Audio capture thread panicked.".to_string())?
        }?;

        if samples.is_empty() {
            return Err(
                "No audio data captured. The recording may have been too short.".into(),
            );
        }

        let duration = self.start_time.elapsed().as_secs_f64();

        // Convert f32 samples to i16 for WAV
        let i16_samples: Vec<i16> = samples
            .iter()
            .map(|&s| (s * 32767.0).clamp(-32768.0, 32767.0) as i16)
            .collect();

        // Write WAV file
        let spec = WavSpec {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(output_path, spec)
            .map_err(|e| format!("Failed to create WAV file: {}", e))?;

        for &sample in &i16_samples {
            writer
                .write_sample(sample)
                .map_err(|e| format!("Failed to write WAV sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV file: {}", e))?;

        Ok(duration)
    }

    /// Pause recording.
    pub fn pause(&self) -> Result<(), String> {
        self.is_paused.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Resume recording after a pause.
    pub fn resume(&self) -> Result<(), String> {
        self.is_paused.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Get the current audio level (RMS) normalized to 0.0–1.0.
    pub fn get_audio_level(&self) -> f32 {
        let samples: Vec<f32> = {
            match self.recent_samples.lock() {
                Ok(buf) => buf.clone(),
                Err(_) => return 0.0,
            }
        };

        if samples.is_empty() {
            return 0.0;
        }

        // RMS calculation
        let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
        let rms = (sum_sq / samples.len() as f32).sqrt();

        // Normalize to 0.0–1.0 range
        // Typical microphone RMS is around 0.01–0.3 for speech
        // We map 0.0–0.5 to 0.0–1.0 for better visual feedback
        (rms * 2.0).min(1.0)
    }

    /// Get the elapsed recording duration in seconds.
    pub fn get_duration(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}

// ── Audio capture thread ───────────────────────────────────────

/// Runs in a dedicated thread. Owns the cpal::Stream and collects samples.
fn run_audio_capture(
    is_paused: Arc<AtomicBool>,
    recent_samples: Arc<Mutex<Vec<f32>>>,
    sample_count: Arc<AtomicU64>,
    stop_rx: std::sync::mpsc::Receiver<()>,
) -> Result<Vec<f32>, String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("No microphone found.")?;

    // Build stream config: we want 16000 Hz, mono, f32
    let supported_config = device
        .supported_input_configs()
        .map_err(|e| format!("Failed to query audio configs: {}", e))?
        .find(|c| {
            c.channels() == CHANNELS
                && c.min_sample_rate().0 <= SAMPLE_RATE
                && c.max_sample_rate().0 >= SAMPLE_RATE
                && c.sample_format() == cpal::SampleFormat::F32
        })
        .or_else(|| {
            device
                .supported_input_configs()
                .ok()?
                .find(|c| {
                    c.min_sample_rate().0 <= SAMPLE_RATE
                        && c.max_sample_rate().0 >= SAMPLE_RATE
                        && c.sample_format() == cpal::SampleFormat::F32
                })
        });

    let config = match supported_config {
        Some(sc) => sc
            .with_sample_rate(cpal::SampleRate(SAMPLE_RATE))
            .config(),
        None => {
            let default_config = device
                .default_input_config()
                .map_err(|e| format!("Failed to get default audio config: {}", e))?;
            default_config.config()
        }
    };

    let actual_sample_rate = config.sample_rate.0;
    let actual_channels = config.channels;

    let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let buffer_clone = buffer.clone();

    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Downmix to mono if needed
                let samples = if actual_channels > 1 {
                    data.chunks(actual_channels as usize)
                        .map(|chunk| chunk.iter().sum::<f32>() / actual_channels as f32)
                        .collect::<Vec<_>>()
                } else {
                    data.to_vec()
                };

                // If sample rate differs, we do simple linear interpolation
                // (for most cases on macOS, we get 16kHz directly)
                let final_samples = if actual_sample_rate != SAMPLE_RATE && actual_sample_rate > 0 {
                    let ratio = SAMPLE_RATE as f64 / actual_sample_rate as f64;
                    let target_len = (samples.len() as f64 * ratio) as usize;
                    (0..target_len)
                        .map(|i| {
                            let src_idx = i as f64 / ratio;
                            let idx = src_idx as usize;
                            if idx < samples.len() {
                                samples[idx]
                            } else {
                                0.0
                            }
                        })
                        .collect::<Vec<_>>()
                } else {
                    samples
                };

                if let Ok(mut buf) = buffer_clone.lock() {
                    buf.extend_from_slice(&final_samples);
                }

                // Update recent samples for audio level
                if let Ok(mut recent) = recent_samples.lock() {
                    recent.extend_from_slice(&final_samples);
                    if recent.len() > RECENT_SAMPLES_COUNT * 2 {
                        let start = recent.len() - RECENT_SAMPLES_COUNT;
                        recent.drain(..start);
                    }
                }

                sample_count.fetch_add(final_samples.len() as u64, Ordering::Relaxed);
            },
            |err| {
                eprintln!("Audio input error: {}", err);
            },
            None,
        )
        .map_err(|e| format!("Failed to build audio stream: {}", e))?;

    stream.play().map_err(|e| format!("Failed to start audio stream: {}", e))?;

    // Wait for stop signal (blocks this thread)
    let _ = stop_rx.recv();

    // Stream is dropped here, releasing the microphone
    drop(stream);

    // Return collected samples
    let samples = buffer
        .lock()
        .map(|buf| buf.clone())
        .unwrap_or_default();

    Ok(samples)
}