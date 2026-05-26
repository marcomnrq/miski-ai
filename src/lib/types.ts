// ── Recording ──────────────────────────────────────────────
export interface RecordingInfo {
  id: string;
  path: string;
  session_name: string;
  started_at: string;
}

export interface AudioLevel {
  rms: number;
  peak: number;
  duration_secs: number;
}

// ── Transcription ──────────────────────────────────────────
export interface Segment {
  start_ms: number;
  end_ms: number;
  text: string;
}

export interface LabelledSegment {
  speaker: string;
  start_ms: number;
  end_ms: number;
  text: string;
}

// ── Meetings ───────────────────────────────────────────────
export interface MeetingSummary {
  id: string;
  title: string;
  created_at: string;
  duration_seconds: number | null;
  language: string;
  is_diarised: boolean;
  folder_id: string | null;
  summary_preview: string | null;
}

export interface Meeting {
  id: string;
  title: string;
  audio_path: string | null;
  transcript_text: string | null;
  diarised_text: string | null;
  summary_markdown: string | null;
  notes: string | null;
  language: string;
  is_diarised: boolean;
  duration_seconds: number | null;
  whisper_model: string | null;
  ollama_model: string | null;
  folder_id: string | null;
  created_at: string;
  updated_at: string;
}

// ── Chat ───────────────────────────────────────────────────
export interface ChatMessage {
  id: string;
  session_id: string;
  role: "user" | "assistant";
  content: string;
  created_at: string;
}

export interface ChatSession {
  id: string;
  title: string | null;
  scope: string;
  folder_id: string | null;
  created_at: string;
  messages: ChatMessage[];
}

// ── Settings ───────────────────────────────────────────────
export interface Settings {
  language: string;
  whisper_model: string;
  ollama_model: string;
  ollama_url: string;
  ai_provider: string;
  remote_ollama_url: string;
  keep_recordings: boolean;
  notifications_enabled: boolean;
  user_name: string | null;
}

export interface SettingsPatch {
  language?: string;
  whisper_model?: string;
  ollama_model?: string;
  ollama_url?: string;
  ai_provider?: string;
  remote_ollama_url?: string;
  keep_recordings?: boolean;
  notifications_enabled?: boolean;
  user_name?: string;
}

// ── Setup ──────────────────────────────────────────────────
export interface SetupStatus {
  ollama_installed: boolean;
  ollama_running: boolean;
  whisper_model_downloaded: boolean;
  ollama_model_pulled: boolean;
  microphone_available: boolean;
}

export interface ModelInfo {
  name: string;
  size: string;
  description: string;
  installed: boolean;
}

export interface PullProgress {
  status: string;
  percent: number;
}

// ── Folders ────────────────────────────────────────────────
export interface Folder {
  id: string;
  name: string;
  icon: string;
  color: string;
  sort_order: number;
  created_at: string;
}