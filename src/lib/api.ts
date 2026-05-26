import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  RecordingInfo, AudioLevel, MeetingSummary, Meeting,
  ChatSession, Settings, SettingsPatch, SetupStatus,
  ModelInfo, PullProgress, Folder
} from "./types";

export const api = {
  // ── Recording ──
  recording: {
    start: (name: string) => invoke<RecordingInfo>("start_recording", { name }),
    stop: () => invoke<string>("stop_recording"),
    pause: () => invoke<void>("pause_recording"),
    resume: () => invoke<void>("resume_recording"),
    getAudioLevel: () => invoke<AudioLevel>("get_audio_level"),
    updateNotes: (notes: string) => invoke<void>("update_recording_notes", { notes }),
  },

  // ── Meetings ──
  meetings: {
    list: () => invoke<MeetingSummary[]>("list_meetings"),
    get: (id: string) => invoke<Meeting>("get_meeting", { id }),
    delete: (id: string) => invoke<void>("delete_meeting", { id }),
    rename: (id: string, name: string) => invoke<void>("rename_meeting", { id, name }),
    saveNotes: (id: string, notes: string) => invoke<void>("save_notes", { id, notes }),
    exportMarkdown: (id: string) => invoke<string>("export_meeting_md", { id }),
    query: (id: string, question: string) => invoke<void>("query_meeting", { id, question }),
    queryAll: (question: string, folderId?: string) =>
      invoke<void>("query_all", { question, folderId: folderId ?? null }),
  },

  // ── Chat ──
  chat: {
    listSessions: () => invoke<ChatSession[]>("list_chat_sessions"),
    getSession: (id: string) => invoke<ChatSession>("get_chat_session", { id }),
    createSession: (scope: string, folderId?: string) =>
      invoke<ChatSession>("create_chat_session", { scope, folderId: folderId ?? null }),
    deleteSession: (id: string) => invoke<void>("delete_chat_session", { id }),
    sendMessage: (sessionId: string, question: string) =>
      invoke<void>("send_chat_message", { sessionId, question }),
    query: (question: string) => invoke<void>("query_all", { question, folderId: null }),
  },

  // ── Settings ──
  settings: {
    get: () => invoke<Settings>("get_settings"),
    update: (patch: SettingsPatch) => invoke<void>("update_settings", { settings: patch }),
    getDataDir: () => invoke<string>("get_data_dir"),
  },

  // ── Setup ──
  setup: {
    check: () => invoke<SetupStatus>("check_setup"),
    listOllamaModels: () => invoke<ModelInfo[]>("list_ollama_models"),
    pullOllamaModel: (name: string) => invoke<void>("pull_ollama_model", { name }),
    downloadWhisperModel: (modelSize: string) => invoke<void>("download_whisper_model", { modelSize }),
  },

  // ── Folders ──
  folders: {
    list: () => invoke<Folder[]>("list_folders"),
    create: (name: string, icon: string, color: string) =>
      invoke<Folder>("create_folder", { name, icon, color }),
    update: (id: string, name: string, icon: string, color: string) =>
      invoke<void>("update_folder", { id, name, icon, color }),
    delete: (id: string) => invoke<void>("delete_folder", { id }),
  },
};

// ── Event Listeners ──────────────────────────────────────────
export const events = {
  onProcessingStatus: (cb: (status: string) => void): Promise<UnlistenFn> =>
    listen<string>("processing-status", (e) => cb(e.payload)),

  onSummaryChunk: (cb: (text: string) => void): Promise<UnlistenFn> =>
    listen<string>("summary-chunk", (e) => cb(e.payload)),

  onSummaryComplete: (cb: () => void): Promise<UnlistenFn> =>
    listen<void>("summary-complete", () => cb()),

  onProcessingComplete: (cb: (meetingId: string) => void): Promise<UnlistenFn> =>
    listen<string>("processing-complete", (e) => cb(e.payload)),

  onQueryChunk: (cb: (text: string) => void): Promise<UnlistenFn> =>
    listen<string>("query-chunk", (e) => cb(e.payload)),

  onQueryComplete: (cb: () => void): Promise<UnlistenFn> =>
    listen<void>("query-complete", () => cb()),

  onWhisperDownloadProgress: (cb: (percent: number) => void): Promise<UnlistenFn> =>
    listen<number>("whisper-download-progress", (e) => cb(e.payload)),

  onWhisperDownloadComplete: (cb: () => void): Promise<UnlistenFn> =>
    listen<void>("whisper-download-complete", () => cb()),

  onModelPullProgress: (cb: (progress: PullProgress) => void): Promise<UnlistenFn> =>
    listen<PullProgress>("model-pull-progress", (e) => cb(e.payload)),

  onModelPullComplete: (cb: () => void): Promise<UnlistenFn> =>
    listen<void>("model-pull-complete", () => cb()),
};