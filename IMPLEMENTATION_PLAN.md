# Miski AI — MVP Implementation Plan

> **Status**: Rust backend is COMPLETE and compiles. Frontend needs to be built.
> **MVP Scope**: Recording, Transcription, Speaker Diarisation, AI Summarization
> **Target**: macOS desktop app using Tauri v2 + React + TypeScript + Tailwind v4

---

## What's Already Done ✅

### Rust Backend (all in `src-tauri/src/`)

| Module | Files | Status |
|--------|-------|--------|
| Data Models | `models.rs` | ✅ Complete — all structs defined |
| Storage | `storage/json_store.rs`, `storage/markdown.rs`, `storage/mod.rs` | ✅ Complete — JSON persistence, markdown export |
| Audio Capture | `audio/recorder.rs`, `audio/mod.rs` | ✅ Complete — cpal-based recording in dedicated thread, 16kHz mono WAV |
| Transcription | `transcription/whisper.rs`, `transcription/diarisation.rs`, `transcription/mod.rs` | ✅ Complete — whisper-rs 0.12 with WhisperState, silence-based speaker diarisation |
| AI Summarization | `ai/ollama.rs`, `ai/prompts.rs`, `ai/mod.rs` | ✅ Complete — Ollama REST client with streaming, structured prompts |
| Processing Pipeline | `pipeline.rs` | ✅ Complete — record → transcribe → diarise → summarize → generate title → save |
| Tauri Commands | `lib.rs` | ✅ Complete — all 25 commands wired up and working |
| App Config | `Cargo.toml`, `capabilities/default.json` | ✅ Complete — all dependencies added |

### Tauri Commands Available

The frontend can call these via `invoke()`:

```typescript
// Recording
invoke('start_recording', { name: string }) → RecordingInfo
invoke('stop_recording') → string (wav path)
invoke('pause_recording') → void
invoke('resume_recording') → void
invoke('get_audio_level') → AudioLevel { rms: f32, peak: f32, duration_secs: f64 }
invoke('update_recording_notes', { notes: string }) → void

// Meetings
invoke('list_meetings') → MeetingSummary[]
invoke('get_meeting', { id: string }) → Meeting
invoke('delete_meeting', { id: string }) → void
invoke('rename_meeting', { id: string, name: string }) → void
invoke('save_notes', { id: string, notes: string }) → void
invoke('export_meeting_md', { id: string }) → string (markdown)

// AI / Streaming
invoke('query_meeting', { id: string, question: string }) → void (emits events)
invoke('query_all', { question: string, folder_id?: string }) → void (emits events)

// Chat
invoke('list_chat_sessions') → ChatSession[]
invoke('create_chat_session', { scope: string, folder_id?: string }) → ChatSession
invoke('delete_chat_session', { id: string }) → void

// Settings
invoke('get_settings') → Settings
invoke('update_settings', { settings: SettingsPatch }) → void

// Setup / Models
invoke('check_setup') → SetupStatus
invoke('list_ollama_models') → ModelInfo[]
invoke('pull_ollama_model', { name: string }) → void (emits events)
invoke('download_whisper_model', { model_size: string }) → void (emits events)

// Folders
invoke('list_folders') → Folder[]
invoke('create_folder', { name: string, icon: string, color: string }) → Folder
invoke('update_folder', { id: string, name: string, icon: string, color: string }) → void
invoke('delete_folder', { id: string }) → void
```

### Tauri Events (listen via `listen()`)

```typescript
listen('processing-status', (e) => string)     // "Transcribing audio...", "Summarizing..."
listen('summary-chunk', (e) => string)          // Streaming summary text chunks
listen('summary-complete', (e) => void)          // Summary finished
listen('query-chunk', (e) => string)             // Streaming Q&A response chunks
listen('query-complete', (e) => void)            // Q&A response finished
listen('processing-complete', (e) => string)     // Meeting ID when pipeline finishes
listen('whisper-download-progress', (e) => u32)  // Download percent
listen('whisper-download-complete', (e) => void)
listen('model-pull-progress', (e) => PullProgress) // { status: string, percent: u32 }
listen('model-pull-complete', (e) => void)
```

---

## What Remains 🚧

All frontend work. The backend is done.

---

## STEP 1: Frontend Dependencies & Configuration

### 1.1 — Install dependencies

File: `miski-ai/package.json`

Add these to `dependencies`:
```json
{
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-shell": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "@tanstack/react-query": "^5",
    "zustand": "^5",
    "react-router-dom": "^7",
    "react-markdown": "^9",
    "remark-gfm": "^4",
    "lucide-react": "^0.468"
  },
  "devDependencies": {
    "@tailwindcss/vite": "^4",
    "tailwindcss": "^4",
    "daisyui": "^5"
  }
}
```

Run: `cd miski-ai && pnpm install`

### 1.2 — Update `vite.config.ts`

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
});
```

### 1.3 — Update `src/globals.css`

```css
@import "tailwindcss";
@plugin "daisyui";

/* Custom theme overrides */
@theme {
  --color-primary: oklch(0.65 0.24 265);
  --color-primary-content: oklch(0.98 0.01 265);
  --color-base: oklch(0.15 0.01 265);
  --color-base-content: oklch(0.95 0.01 265);
  --color-accent: oklch(0.75 0.18 160);
}

/* Dark mode is default */
@media (prefers-color-scheme: dark) {
  :root {
    color-scheme: dark;
  }
}
```

### 1.4 — Update `index.html`

Change the title to `Miski AI`:
```html
<title>Miski AI</title>
```

---

## STEP 2: TypeScript Types

### 2.1 — Create `src/lib/types.ts`

This file MUST mirror the Rust models exactly:

```typescript
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
export interface ChatSession {
  id: string;
  title: string | null;
  scope: string;
  folder_id: string | null;
  created_at: string;
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
```

---

## STEP 3: API Layer

### 3.1 — Create `src/lib/api.ts`

```typescript
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
    createSession: (scope: string, folderId?: string) =>
      invoke<ChatSession>("create_chat_session", { scope, folderId: folderId ?? null }),
    deleteSession: (id: string) => invoke<void>("delete_chat_session", { id }),
  },

  // ── Settings ──
  settings: {
    get: () => invoke<Settings>("get_settings"),
    update: (patch: SettingsPatch) => invoke<void>("update_settings", { settings: patch }),
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
  onProcessingStatus: (cb: (status: string) => void) =>
    listen<string>("processing-status", (e) => cb(e.payload)),

  onSummaryChunk: (cb: (text: string) => void) =>
    listen<string>("summary-chunk", (e) => cb(e.payload)),

  onSummaryComplete: (cb: () => void) =>
    listen<void>("summary-complete", () => cb()),

  onProcessingComplete: (cb: (meetingId: string) => void) =>
    listen<string>("processing-complete", (e) => cb(e.payload)),

  onQueryChunk: (cb: (text: string) => void) =>
    listen<string>("query-chunk", (e) => cb(e.payload)),

  onQueryComplete: (cb: () => void) =>
    listen<void>("query-complete", () => cb()),

  onWhisperDownloadProgress: (cb: (percent: number) => void) =>
    listen<number>("whisper-download-progress", (e) => cb(e.payload)),

  onWhisperDownloadComplete: (cb: () => void) =>
    listen<void>("whisper-download-complete", () => cb()),

  onModelPullProgress: (cb: (progress: PullProgress) => void) =>
    listen<PullProgress>("model-pull-progress", (e) => cb(e.payload)),

  onModelPullComplete: (cb: () => void) =>
    listen<void>("model-pull-complete", () => cb()),
};
```

---

## STEP 4: Zustand Stores

### 4.1 — Create `src/stores/recordingStore.ts`

```typescript
import { create } from "zustand";

interface RecordingStore {
  sessionName: string;
  notes: string;
  setSessionName: (name: string) => void;
  setNotes: (notes: string) => void;
  reset: () => void;
}

export const useRecordingStore = create<RecordingStore>((set) => ({
  sessionName: `Meeting ${new Date().toLocaleDateString()}`,
  notes: "",
  setSessionName: (name) => set({ sessionName: name }),
  setNotes: (notes) => set({ notes }),
  reset: () =>
    set({
      sessionName: `Meeting ${new Date().toLocaleDateString()}`,
      notes: "",
    }),
}));
```

---

## STEP 5: Custom Hooks

### 5.1 — Create `src/hooks/useRecording.ts`

```typescript
import { useState, useEffect, useCallback, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { api, events } from "../lib/api";
import { useRecordingStore } from "../stores/recordingStore";

export type RecordingState =
  | "idle"
  | "recording"
  | "paused"
  | "processing"
  | "done";

export function useRecording() {
  const [state, setState] = useState<RecordingState>("idle");
  const [meetingId, setMeetingId] = useState<string | null>(null);
  const [audioLevel, setAudioLevel] = useState(0);
  const [duration, setDuration] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const levelIntervalRef = useRef<number | null>(null);
  const navigate = useNavigate();
  const store = useRecordingStore();

  const start = useCallback(
    async (name?: string) => {
      try {
        setError(null);
        const sessionName =
          name || store.sessionName || `Meeting ${new Date().toLocaleDateString()}`;
        await api.recording.start(sessionName);
        setState("recording");
        navigate("/recorder");

        // Start polling audio level
        levelIntervalRef.current = window.setInterval(async () => {
          try {
            const level = await api.recording.getAudioLevel();
            setAudioLevel(level.rms);
            setDuration(level.duration_secs);
          } catch {
            // ignore polling errors
          }
        }, 100);
      } catch (e: any) {
        setError(e.toString());
      }
    },
    [navigate, store.sessionName]
  );

  const stop = useCallback(async () => {
    try {
      // Stop level polling
      if (levelIntervalRef.current) {
        clearInterval(levelIntervalRef.current);
        levelIntervalRef.current = null;
      }
      // Save notes before stopping
      if (store.notes) {
        await api.recording.updateNotes(store.notes);
      }

      setState("processing");
      setAudioLevel(0);
      navigate("/processing");

      await api.recording.stop();
      // The processing-complete event handler will update state
    } catch (e: any) {
      setError(e.toString());
      setState("idle");
    }
  }, [navigate, store.notes]);

  const pause = useCallback(async () => {
    try {
      await api.recording.pause();
      setState("paused");
    } catch (e: any) {
      setError(e.toString());
    }
  }, []);

  const resume = useCallback(async () => {
    try {
      await api.recording.resume();
      setState("recording");
    } catch (e: any) {
      setError(e.toString());
    }
  }, []);

  // Listen for processing-complete
  useEffect(() => {
    const unlisten = events.onProcessingComplete((id) => {
      setMeetingId(id);
      setState("done");
      navigate(`/meetings/${id}`);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [navigate]);

  // Cleanup interval on unmount
  useEffect(() => {
    return () => {
      if (levelIntervalRef.current) {
        clearInterval(levelIntervalRef.current);
      }
    };
  }, []);

  return {
    state,
    meetingId,
    audioLevel,
    duration,
    error,
    start,
    stop,
    pause,
    resume,
  };
}
```

### 5.2 — Create `src/hooks/useMeetings.ts`

```typescript
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import type { MeetingSummary, Meeting } from "../lib/types";

export function useMeetings() {
  return useQuery<MeetingSummary[]>({
    queryKey: ["meetings"],
    queryFn: () => api.meetings.list(),
  });
}

export function useMeeting(id: string) {
  return useQuery<Meeting>({
    queryKey: ["meetings", id],
    queryFn: () => api.meetings.get(id),
    enabled: !!id,
  });
}

export function useDeleteMeeting() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.meetings.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["meetings"] });
    },
  });
}

export function useRenameMeeting() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) =>
      api.meetings.rename(id, name),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: ["meetings"] });
      queryClient.invalidateQueries({ queryKey: ["meetings", id] });
    },
  });
}

export function useSaveNotes() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, notes }: { id: string; notes: string }) =>
      api.meetings.saveNotes(id, notes),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: ["meetings", id] });
    },
  });
}
```

### 5.3 — Create `src/hooks/useStreaming.ts`

```typescript
import { useState, useCallback, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export function useStreaming() {
  const [text, setText] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const unlisteners = useRef<UnlistenFn[]>([]);

  const startStream = useCallback(
    async (
      chunkEvent: "summary-chunk" | "query-chunk",
      doneEvent: "summary-complete" | "query-complete"
    ) => {
      setText("");
      setIsStreaming(true);

      const unlistenChunk = await listen<string>(chunkEvent, (e) => {
        setText((prev) => prev + e.payload);
      });

      const unlistenDone = await listen<void>(doneEvent, () => {
        setIsStreaming(false);
        // Cleanup listeners
        unlistenChunk();
        unlistenDone();
        unlisteners.current = unlisteners.current.filter(
          (fn) => fn !== unlistenChunk && fn !== unlistenDone
        );
      });

      unlisteners.current.push(unlistenChunk, unlistenDone);
    },
    []
  );

  const reset = useCallback(() => {
    setText("");
    setIsStreaming(false);
    unlisteners.current.forEach((fn) => fn());
    unlisteners.current = [];
  }, []);

  return { text, isStreaming, startStream, reset };
}
```

### 5.4 — Create `src/hooks/useSettings.ts`

```typescript
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import type { Settings, SettingsPatch } from "../lib/types";

export function useSettings() {
  return useQuery<Settings>({
    queryKey: ["settings"],
    queryFn: () => api.settings.get(),
  });
}

export function useUpdateSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (patch: SettingsPatch) => api.settings.update(patch),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
  });
}
```

### 5.5 — Create `src/hooks/useSetup.ts`

```typescript
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import type { SetupStatus } from "../lib/types";

export function useSetup() {
  return useQuery<SetupStatus>({
    queryKey: ["setup"],
    queryFn: () => api.setup.check(),
    staleTime: 0, // Always refetch
  });
}
```

---

## STEP 6: Layout Components

### 6.1 — Create `src/components/layout/Sidebar.tsx`

A vertical sidebar with navigation links. Uses DaisyUI + Tailwind.

**Requirements:**
- Logo/brand at top: "Miski AI" with a microphone icon
- Navigation links using `react-router-dom`'s `NavLink`:
  - Home (`/`) — icon: `Home` from lucide-react
  - Record (`/recorder`) — icon: `Mic` from lucide-react
  - Meetings (`/meetings`) — icon: `FileText` from lucide-react
  - Chat (`/chat`) — icon: `MessageSquare` from lucide-react
  - Settings (`/settings`) — icon: `Settings` from lucide-react
- Active route highlighted with `bg-primary/20 text-primary`
- Version number at bottom: "v0.1.0"
- Width: `w-56`, height: `h-screen`, `bg-base-200`
- Use `btn-ghost` for nav items, full width

### 6.2 — Create `src/components/layout/AppShell.tsx`

```tsx
import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";

export function AppShell({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-screen bg-base-100">
      <Sidebar />
      <main className="flex-1 overflow-auto">
        {children}
      </main>
    </div>
  );
}
```

---

## STEP 7: Page Components

### 7.1 — `src/pages/HomePage.tsx`

**Purpose**: Welcome screen with quick-start recording and recent meetings.

**Layout:**
- Top section: App name "Miski AI" with tagline "Privacy-first meeting recorder"
- Center: Large "Start Recording" button (use `btn-primary btn-lg`) with Mic icon
- Below: "Recent Meetings" heading
- List of last 5 meetings from `useMeetings()` hook
- Each meeting shows: title, date (formatted with `toLocaleDateString`), duration, summary preview (first 100 chars)
- Click meeting card → navigate to `/meetings/:id`
- If no meetings: "No meetings yet. Start recording to create your first one!"

### 7.2 — `src/pages/RecorderPage.tsx`

**Purpose**: Main recording interface with live audio visualization.

**Layout (vertical, centered):**
1. Session name input (editable text input, value from `useRecordingStore`)
2. Large timer display: `MM:SS` format, monospace font, `text-5xl`
3. Audio waveform: 8 animated bars using `audioLevel` from `useRecording()`
   - Bar height: `h-${Math.max(8, level * 48)}` based on audio level
   - Color: `bg-primary` when recording, `bg-base-300` when paused
   - Animate with CSS transitions (`transition-all duration-100`)
4. Control buttons (horizontal row):
   - Pause/Resume toggle button (icon: `Pause`/`Play`)
   - Stop button (icon: `Square`, red/danger color)
5. Collapsible notes section:
   - Toggle button "Notes" with `ChevronDown` icon
   - When expanded: textarea for notes, updates `useRecordingStore`
6. Only visible when `state === 'recording'` or `state === 'paused'`
7. If `state === 'idle'`, show a "Start Recording" button that calls `start()`

### 7.3 — `src/pages/ProcessingPage.tsx`

**Purpose**: Shows processing progress while the pipeline runs.

**Layout:**
- Centered vertically and horizontally
- Spinning loading animation (use DaisyUI `loading loading-spinner loading-lg`)
- Status text: listen to `processing-status` events
  - "Transcribing audio..."
  - "Identifying speakers..."
  - "Generating summary..."
  - "Saving meeting..."
- Below: Live streaming summary text as it arrives
  - Listen to `summary-chunk` events using `useStreaming()`
  - Render with `react-markdown` + `remark-gfm`
- Auto-navigates to meeting page on `processing-complete`

### 7.4 — `src/pages/MeetingPage.tsx`

**Purpose**: View a single meeting's details — summary, transcript, notes, Q&A.

**Layout:**
- Header: Title (editable inline — click to edit, blur to save via `useRenameMeeting`)
- Metadata row: date, duration (formatted), language badge, model used
- Tab navigation (use DaisyUI `tabs`):
  - **Summary**: Render `summary_markdown` with `react-markdown` + `remark-gfm`
  - **Transcript**: 
    - If `is_diarised`: parse `diarised_text` which has format `[Speaker A] text`
    - Show speaker labels as colored badges
    - Different speakers get different colors
    - If not diarised: show plain `transcript_text`
  - **Notes**: Editable textarea, auto-save on blur via `useSaveNotes`
- Bottom fixed bar: AskBar component
  - Input field + send button
  - On submit: call `api.meetings.query(id, question)`
  - Display streaming response below using `useStreaming()`
- Action buttons (top right): Export Markdown (saves file), Delete

### 7.5 — `src/pages/MeetingsPage.tsx`

**Purpose**: List all meetings with search/filter.

**Layout:**
- Search input at top (filters by title)
- Grid of meeting cards (2-3 columns)
- Each card shows: title, date, duration, language, summary preview (150 chars), diarised badge
- Click → navigate to `/meetings/:id`
- Empty state: "No meetings yet" with link to start recording
- Sort by date (newest first)

### 7.6 — `src/pages/ChatPage.tsx`

**Purpose**: Q&A across multiple meetings.

**Layout:**
- Left panel: list of previous chat sessions (optional for MVP)
- Main area: Chat messages (user + assistant)
- Input bar at bottom
- On submit: call `api.meetings.queryAll(question)`
- Show streaming responses
- Each message: avatar icon, text content (markdown rendered), timestamp

### 7.7 — `src/pages/SettingsPage.tsx`

**Purpose**: Configure app settings.

**Form fields:**
- Language: select dropdown with options: auto, en, es, fr, de, pt, it, ja, ko, zh
- Whisper model: select dropdown with options: tiny, base, small, medium, large
  - Show download button if not downloaded, progress bar during download
  - Listen to `whisper-download-progress` for progress
- Ollama URL: text input (default: `http://localhost:11434`)
- Ollama model: select dropdown (fetched from `list_ollama_models()`)
  - Show pull button if model not available, progress during pull
  - Listen to `model-pull-progress` for progress
- Keep recordings: toggle switch
- Notifications: toggle switch
- User name: text input
- Save button: calls `useUpdateSettings()`
- Setup status shown at top: green checkmarks for OK items, red X for missing

---

## STEP 8: App Router

### 8.1 — Rewrite `src/App.tsx`

```tsx
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AppShell } from "./components/layout/AppShell";
import { HomePage } from "./pages/HomePage";
import { RecorderPage } from "./pages/RecorderPage";
import { ProcessingPage } from "./pages/ProcessingPage";
import { MeetingPage } from "./pages/MeetingPage";
import { MeetingsPage } from "./pages/MeetingsPage";
import { ChatPage } from "./pages/ChatPage";
import { SettingsPage } from "./pages/SettingsPage";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { retry: false, refetchOnWindowFocus: false },
  },
});

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <AppShell>
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/recorder" element={<RecorderPage />} />
            <Route path="/processing" element={<ProcessingPage />} />
            <Route path="/meetings" element={<MeetingsPage />} />
            <Route path="/meetings/:id" element={<MeetingPage />} />
            <Route path="/chat" element={<ChatPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>
        </AppShell>
      </BrowserRouter>
    </QueryClientProvider>
  );
}
```

### 8.2 — Clean up `src/main.tsx`

Make sure it just renders App:
```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./globals.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

Delete `src/App.css` — it's not needed.

---

## STEP 9: Delete Old Files

Remove these files that are no longer needed:
- `src/App.css` — replaced by Tailwind
- `src/assets/react.svg` — not used

---

## Implementation Order (Priority)

The implementing AI agent should follow this exact order:

1. **STEP 1**: Install deps + configure Vite + Tailwind + globals.css
2. **STEP 2**: Create `src/lib/types.ts` — TypeScript types
3. **STEP 3**: Create `src/lib/api.ts` — Tauri invoke wrappers
4. **STEP 4**: Create `src/stores/recordingStore.ts` — Zustand store
5. **STEP 5**: Create all hooks in `src/hooks/`
6. **STEP 6**: Create layout components (Sidebar, AppShell)
7. **STEP 8**: Rewrite `src/App.tsx` with router + clean `main.tsx`
8. **STEP 9**: Delete old files
9. **STEP 7.1**: Create `HomePage.tsx`
10. **STEP 7.2**: Create `RecorderPage.tsx` ← **Most critical for MVP**
11. **STEP 7.3**: Create `ProcessingPage.tsx`
12. **STEP 7.4**: Create `MeetingPage.tsx` ← **Second most critical**
13. **STEP 7.5**: Create `MeetingsPage.tsx`
14. **STEP 7.6**: Create `ChatPage.tsx` (simplified for MVP)
15. **STEP 7.7**: Create `SettingsPage.tsx`

---

## Key Design Decisions for Implementer

1. **Use DaisyUI components extensively** — `btn`, `card`, `input`, `textarea`, `tabs`, `badge`, `loading`, `tooltip`. This is our component library.
2. **Dark mode is the default** — the app looks best in dark mode.
3. **All data flows through `api.ts`** — never call `invoke()` directly in components.
4. **React Query for all server state** — meetings, settings, folders. Only recording ephemeral state goes to Zustand.
5. **Streaming via Tauri events** — listen for chunk events, accumulate text in `useStreaming` hook.
6. **No need for complex state management** — React Query + Zustand for recording state is sufficient.
7. **Error handling**: show errors in `alert()` or a simple toast. Don't over-engineer for MVP.
8. **Keep components simple** — each page is a single file. Extract sub-components only if reused.

---

## Testing the MVP

After implementation, test this flow:
1. Open app → see HomePage
2. Click "Start Recording" → see RecorderPage with timer + waveform
3. Speak into microphone for 10+ seconds
4. Click Stop → see ProcessingPage with "Transcribing..."
5. Wait for processing → auto-navigate to MeetingPage
6. See Summary tab with structured markdown
7. See Transcript tab with speaker labels (Speaker A, Speaker B, etc.)
8. Use AskBar to ask a question → see streaming response
9. Go to MeetingsPage → see the meeting in the list
10. Go to SettingsPage → verify Ollama connection, model status