# Miski AI — MVP Completion Plan

## Current State Summary

### ✅ What's Done
- **Rust backend**: All 26 Tauri commands compile (`cargo check` passes with only warnings)
- **Frontend**: All 7 pages + layout + hooks + stores + types + API layer compile cleanly (`tsc --noEmit` zero errors, `vite build` succeeds)
- **Storage**: JSON persistence layer with all methods
- **Audio**: cpal-based recorder with dedicated thread
- **Transcription**: whisper-rs 0.12 integration
- **AI**: Ollama REST client with streaming

### ❌ What Needs Work for a Functional MVP
The code compiles but won't run end-to-end. The issues are:
1. `stop_recording` has a borrow-after-drop bug that will deadlock at runtime
2. Frontend has no error boundaries — backend failures cause blank screens
3. Processing pipeline events aren't fully connected to frontend navigation
4. No dark mode toggle despite dark CSS vars being defined
5. Chat sessions don't persist messages
6. Missing loading states and empty states throughout
7. The processing-to-meeting navigation flow is broken

---

## Phase 1: Critical Rust Backend Fixes

### 1.1 Fix `stop_recording` Deadlock

**File**: `src-tauri/src/lib.rs`  
**Problem**: The function borrows `recorder` from `recorder_guard`, calls `stop()`, then drops the guard and re-locks to `take()`. This technically compiles but the pattern is fragile — if `stop()` panics, the MutexGuard is held across the panic.

**Fix**: Take the recorder directly and stop it:

```rust
#[tauri::command]
async fn stop_recording(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 1. Take the recorder out of state (moves it out of the Option)
    let recorder = state
        .recorder
        .lock()
        .map_err(|e| e.to_string())?
        .take()
        .ok_or("Not recording")?;

    // 2. Take metadata
    let meta = state
        .recording_meta
        .lock()
        .map_err(|e| e.to_string())?
        .take()
        .ok_or("No recording metadata")?;

    let wav_path = state.store.recordings_dir().join(format!("{}.wav", meta.id));

    // 3. Stop the recorder — saves WAV file (blocks until audio thread finishes)
    //    NOTE: This is intentionally blocking. The MutexGuards were already dropped
    //    above since we .take()'d the values out.
    let _duration = recorder.stop(&wav_path)?;

    // 4. Emit processing-started event
    let _ = app.emit("processing-status", "Transcribing audio...");

    // 5. Clone what we need for the async pipeline task
    let store = state.store.clone();
    let wav_path_clone = wav_path.clone();
    let session_name = meta.session_name.clone();
    let notes = if meta.notes.is_empty() {
        None
    } else {
        Some(meta.notes)
    };

    // 6. Spawn the processing pipeline in a background task
    tauri::async_runtime::spawn(async move {
        match pipeline::process_recording(
            &wav_path_clone,
            &session_name,
            notes.as_deref(),
            app.clone(),
            &store,
        )
        .await
        {
            Ok(meeting) => {
                let _ = app.emit("processing-complete", meeting.id.clone());
                let _ = app.emit("processing-status", "Complete!");
            }
            Err(e) => {
                eprintln!("Processing pipeline error: {}", e);
                let _ = app.emit("processing-status", format!("Error: {}", e));
            }
        }
    });

    Ok(wav_path.to_string_lossy().to_string())
}
```

**IMPORTANT**: Verify that `AudioRecorderHandle::stop(&self, path: &Path)` takes `&self` (not `self`). If it takes `self` (ownership), the fix is even simpler — just call `recorder.stop(...)` directly after `take()`. Check `src-tauri/src/audio/recorder.rs` for the actual signature and adjust accordingly.

### 1.2 Ensure Pipeline Emits All Events

**File**: `src-tauri/src/pipeline.rs`

The pipeline MUST emit these events in order:
1. `"processing-status"` with string messages: `"Transcribing audio..."`, `"Identifying speakers..."`, `"Generating summary..."`, `"Saving meeting..."`
2. `"summary-chunk"` with partial summary text (streaming from Ollama)
3. `"summary-complete"` (no payload) when summary finishes
4. `"processing-complete"` with the meeting ID string

Check the pipeline file and ensure every stage emits a status update. If any stage is missing an emit, add one:

```rust
// Example pattern for each pipeline stage:
let _ = app.emit("processing-status", "Transcribing audio...");
let transcript = whisper.transcribe(&wav_path, &settings.language)?;
let _ = app.emit("processing-status", "Identifying speakers...");
let turns = diarise(&transcript, settings.silence_threshold, settings.silence_duration_ms);
let _ = app.emit("processing-status", "Generating summary...");
// ... streaming summary emits summary-chunk events ...
let _ = app.emit("summary-complete", ());
let _ = app.emit("processing-status", "Saving meeting...");
let meeting = store.save_meeting(&meeting_data)?;
let _ = app.emit("processing-complete", meeting.id.clone());
```

### 1.3 Verify `AudioRecorderHandle` Thread Safety

**File**: `src-tauri/src/audio/recorder.rs`

Ensure:
- `stop()` method properly signals the audio thread to stop and joins it
- The WAV file is fully written before `stop()` returns
- `get_audio_level()` returns 0.0 when not recording (not an error)
- `get_duration()` returns elapsed seconds

If `stop()` takes `&self`, ensure interior mutability (e.g., using `Mutex` or `AtomicBool` for the stop flag). If it takes `self`, update `lib.rs` accordingly.

### 1.4 Verify Ollama Client Streaming

**File**: `src-tauri/src/ai/ollama.rs`

Ensure:
- `query_transcript_streaming()` and `query_corpus_streaming()` emit `"query-chunk"` events for each chunk and `"query-complete"` at the end
- The HTTP client handles connection failures gracefully (Ollama not running)
- `health_check()` returns `false` on any error (not an error propagation)
- `list_models()` parses the Ollama API response correctly: `{"models": [{"name": "llama3:latest", ...}]}`

### 1.5 Verify Cargo.toml Dependencies

Make sure these are in `src-tauri/Cargo.toml`:
```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
cpal = "0.15"
hound = "3"
whisper-rs = { version = "0.12", features = ["whisper-cpp-log"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
futures-util = "0.3"
anyhow = "1"
thiserror = "2"
dirs = "5"
```

---

## Phase 2: Frontend Critical Fixes

### 2.1 Fix Processing → Meeting Navigation

**File**: `src/pages/ProcessingPage.tsx`

The processing page must:
1. Listen for `"processing-complete"` event to get the meeting ID
2. Navigate to `/meetings/{id}` when complete
3. Show error state and allow going back to home if processing fails

```tsx
// In ProcessingPage.tsx, add this to the streaming hook setup:
useEffect(() => {
  const unlisten = events.onProcessingComplete(async (meetingId) => {
    setProcessingStatus("Complete!");
    // Navigate to the meeting detail page
    navigate(`/meetings/${meetingId}`);
  });
  return () => { unlisten.then(fn => fn()); };
}, [navigate]);
```

Check the current `ProcessingPage.tsx` and ensure it has this navigation logic.

### 2.2 Add Error Boundaries

**File**: Create `src/components/ErrorBoundary.tsx`

```tsx
import { Component, type ReactNode } from "react";
import { AlertCircle } from "lucide-react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex items-center justify-center h-full">
          <div className="text-center space-y-4 max-w-md">
            <AlertCircle size={48} className="text-[var(--danger)] mx-auto" />
            <h2 className="text-xl font-bold text-[var(--text-primary)]">
              Something went wrong
            </h2>
            <p className="text-sm text-[var(--text-secondary)]">
              {this.state.error?.message ?? "An unexpected error occurred."}
            </p>
            <button
              onClick={() => window.location.reload()}
              className="bg-[var(--accent)] text-white px-4 py-2 rounded-lg hover:bg-[var(--accent-hover)] transition-colors cursor-pointer"
            >
              Reload
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
```

Then wrap the app in `App.tsx`:
```tsx
import { ErrorBoundary } from "./components/ErrorBoundary";
// ... inside the render:
<ErrorBoundary>
  <QueryClientProvider client={queryClient}>
    <RouterProvider router={router} />
  </QueryClientProvider>
</ErrorBoundary>
```

### 2.3 Add Dark Mode Toggle

**File**: `src/components/layout/Sidebar.tsx`

Add a theme toggle button at the bottom of the sidebar:

```tsx
import { Moon, Sun } from "lucide-react";

// Inside the Sidebar component:
const [dark, setDark] = useState(() => 
  document.documentElement.classList.contains("dark")
);

const toggleTheme = () => {
  const next = !dark;
  setDark(next);
  document.documentElement.classList.toggle("dark", next);
};

// At the bottom of the sidebar nav, before the settings link:
<button
  onClick={toggleTheme}
  className="flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-[var(--text-secondary)] hover:bg-[var(--surface-raised)] transition-colors w-full cursor-pointer"
>
  {dark ? <Sun size={18} /> : <Moon size={18} />}
  <span>{dark ? "Light Mode" : "Dark Mode"}</span>
</button>
```

### 2.4 Improve RecorderPage — Add Timer Display

**File**: `src/pages/RecorderPage.tsx`

The recorder page should show elapsed time during recording. The `get_audio_level` command returns `duration_secs`. Use it:

```tsx
// In the recorder page, where audio level is polled:
const formatTime = (secs: number) => {
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
};

// Display the timer:
<div className="text-4xl font-mono text-[var(--text-primary)]">
  {formatTime(audioLevel?.duration_secs ?? 0)}
</div>
```

### 2.5 Fix Empty States

All list pages need empty states:

**MeetingsPage.tsx** — when no meetings:
```tsx
{meetings.length === 0 && (
  <div className="flex flex-col items-center justify-center py-20 text-center">
    <FileText size={48} className="text-[var(--text-secondary)] mb-4" />
    <h3 className="text-lg font-medium text-[var(--text-primary)]">No meetings yet</h3>
    <p className="text-sm text-[var(--text-secondary)] mt-1">
      Record your first meeting to get started
    </p>
    <Link
      to="/record"
      className="mt-4 bg-[var(--accent)] text-white px-4 py-2 rounded-lg hover:bg-[var(--accent-hover)] transition-colors"
    >
      Start Recording
    </Link>
  </div>
)}
```

**ChatPage.tsx** — when no sessions:
```tsx
{sessions.length === 0 && (
  <div className="flex flex-col items-center justify-center py-20 text-center">
    <MessageSquare size={48} className="text-[var(--text-secondary)] mb-4" />
    <h3 className="text-lg font-medium text-[var(--text-primary)]">No chat sessions</h3>
    <p className="text-sm text-[var(--text-secondary)] mt-1">
      Ask a question about your meetings to get started
    </p>
  </div>
)}
```

### 2.6 Chat Session Message Persistence

**Problem**: The `ChatSession` model only has metadata (id, title, scope, folder_id). Individual messages (question + answer pairs) are not stored. For MVP, we need at least basic message storage.

**Option A (Simplest)**: Store messages in the chat session JSON file. Add a `messages` field to `ChatSession`:

**Rust `models.rs`**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,       // "user" or "assistant"
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: Option<String>,
    pub scope: String,
    pub folder_id: Option<String>,
    pub created_at: String,
    pub messages: Vec<ChatMessage>,
}
```

**Rust `lib.rs`** — Add a command to append a message:
```rust
#[tauri::command]
async fn send_chat_message(
    session_id: String,
    question: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut sessions = state.store.list_chat_sessions().map_err(|e| e.to_string())?;
    let session = sessions.iter_mut()
        .find(|s| s.id == session_id)
        .ok_or("Session not found")?;

    // Add user message
    session.messages.push(ChatMessage {
        role: "user".into(),
        content: question.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    });

    // Update title from first message if not set
    if session.title.is_none() {
        session.title = Some(question.chars().take(50).collect());
    }

    // Save with user message
    state.store.save_chat_session(session, &session.messages).map_err(|e| e.to_string())?;

    // Now stream the AI response
    let settings = state.store.get_settings().map_err(|e| e.to_string())?;
    let client = OllamaClient::new(&settings.ollama_url, &settings.ollama_model);

    // Build context based on session scope
    // ... query appropriate meetings ...

    // After streaming completes, save assistant message
    // This needs to be done in the streaming callback or after query_chunk events

    Ok(())
}
```

**TypeScript `types.ts`** — Update:
```typescript
export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
  timestamp: string;
}

export interface ChatSession {
  id: string;
  title: string | null;
  scope: string;
  folder_id: string | null;
  created_at: string;
  messages: ChatMessage[];
}
```

---

## Phase 3: Integration & Polish

### 3.1 First-Run Setup Flow

When the app starts for the first time, the user needs:
1. Whisper model downloaded
2. Ollama running with a model pulled
3. Microphone permission granted

**File**: `src/pages/HomePage.tsx`

Add a setup banner when setup is incomplete:

```tsx
import { useSetup } from "../hooks/useSetup";

export function HomePage() {
  const { data: setup } = useSetup();
  
  const needsSetup = setup && (
    !setup.whisper_model_downloaded || 
    !setup.ollama_running || 
    !setup.ollama_model_pulled
  );

  return (
    <div>
      {needsSetup && (
        <div className="bg-[var(--warning)]/10 border border-[var(--warning)]/30 rounded-xl p-4 mb-6 flex items-center gap-3">
          <AlertTriangle size={20} className="text-[var(--warning)]" />
          <div>
            <p className="font-medium text-[var(--text-primary)]">Setup incomplete</p>
            <p className="text-sm text-[var(--text-secondary)]">
              Some components are missing. Visit{" "}
              <Link to="/settings" className="text-[var(--accent)] underline">
                Settings
              </Link>{" "}
              to complete setup.
            </p>
          </div>
        </div>
      )}
      {/* ... rest of home page */}
    </div>
  );
}
```

### 3.2 Whisper Model Download Progress

**File**: `src/pages/SettingsPage.tsx`

Add a download button with progress bar for the Whisper model:

```tsx
const [downloadProgress, setDownloadProgress] = useState<number | null>(null);
const [downloading, setDownloading] = useState(false);

// Listen for download progress
useEffect(() => {
  const unlisten1 = events.onWhisperDownloadProgress((percent) => {
    setDownloadProgress(percent);
  });
  const unlisten2 = events.onWhisperDownloadComplete(() => {
    setDownloading(false);
    setDownloadProgress(null);
    // Invalidate setup to refresh status
    queryClient.invalidateQueries({ queryKey: ["setup"] });
  });
  return () => {
    unlisten1.then(fn => fn());
    unlisten2.then(fn => fn());
  };
}, []);

// In the setup status section, add download button:
{!setup.whisper_model_downloaded && (
  <button
    onClick={async () => {
      setDownloading(true);
      try {
        await api.setup.downloadWhisperModel(form.whisper_model ?? "base");
      } catch (e) {
        setDownloading(false);
      }
    }}
    disabled={downloading}
    className="bg-[var(--accent)] text-white px-4 py-2 rounded-lg text-sm hover:bg-[var(--accent-hover)] disabled:opacity-50 cursor-pointer"
  >
    {downloading ? `Downloading... ${downloadProgress ?? 0}%` : "Download Whisper Model"}
  </button>
)}

{downloadProgress !== null && (
  <div className="w-full bg-[var(--surface-raised)] rounded-full h-2 mt-2">
    <div
      className="bg-[var(--accent)] h-2 rounded-full transition-all"
      style={{ width: `${downloadProgress}%` }}
    />
  </div>
)}
```

### 3.3 Ollama Model Pull Progress

Similar pattern to whisper download, using `onModelPullProgress` and `onModelPullComplete` events:

```tsx
const [pullProgress, setPullProgress] = useState<PullProgress | null>(null);

// Listen for pull progress
useEffect(() => {
  const unlisten1 = events.onModelPullProgress((progress) => {
    setPullProgress(progress);
  });
  const unlisten2 = events.onModelPullComplete(() => {
    setPullProgress(null);
    queryClient.invalidateQueries({ queryKey: ["setup"] });
  });
  return () => {
    unlisten1.then(fn => fn());
    unlisten2.then(fn => fn());
  };
}, []);

// Button to pull model:
{!setup.ollama_model_pulled && setup.ollama_running && (
  <button
    onClick={async () => {
      try {
        await api.setup.pullOllamaModel(form.ollama_model ?? "llama3");
      } catch (e) {
        console.error(e);
      }
    }}
    className="bg-[var(--accent)] text-white px-4 py-2 rounded-lg text-sm hover:bg-[var(--accent-hover)] cursor-pointer"
  >
    Pull {form.ollama_model ?? "llama3"}
  </button>
)}
```

### 3.4 Microphone Permission (macOS)

On macOS, the app needs microphone permission. Tauri v2 uses the `tauri-plugin-opener` but microphone access needs a specific capability.

**File**: `src-tauri/capabilities/default.json`

Ensure it includes:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default"
  ]
}
```

The microphone permission on macOS is handled by the OS — cpal will trigger the system permission dialog on first use. If denied, `check_setup` will report `microphone_available: false`.

### 3.5 App Metadata

**File**: `src-tauri/tauri.conf.json`

Update the app name and window settings:
```json
{
  "productName": "Miski AI",
  "version": "0.1.0",
  "identifier": "com.miski-ai.app",
  "app": {
    "windows": [
      {
        "title": "Miski AI",
        "width": 1200,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600
      }
    ]
  }
}
```

---

## Phase 4: End-to-End Test Plan

After implementing the above, manually test this flow:

1. **First Launch**: App opens → HomePage shows setup banner → click Settings
2. **Settings**: Shows warning for missing whisper model → click Download → progress bar → completes
3. **Start Ollama**: `ollama serve` in terminal → Settings shows Ollama running → pull model
4. **Record**: Click Record in sidebar → name the recording → Start → see timer + audio level → Stop
5. **Processing**: Auto-navigate to processing page → see status updates → auto-navigate to meeting
6. **Meeting Detail**: See transcript, summary, speakers → edit notes → export markdown
7. **Meetings List**: See the meeting in list → search → click to view
8. **Chat**: Go to Chat → ask question about meeting → see streaming response
9. **Dark Mode**: Toggle dark mode in sidebar → all pages render correctly
10. **Error Case**: Record without whisper model → see error message, not blank screen

---

## Implementation Order (Priority)

1. **Fix `stop_recording` in Rust** (1.1) — Without this, recording doesn't work
2. **Verify pipeline events** (1.2) — Without this, processing page is stuck
3. **Fix Processing → Meeting navigation** (2.1) — Without this, user is stuck on processing page
4. **Add ErrorBoundary** (2.2) — Prevents blank screens
5. **First-run setup banner** (3.1) — Guides new users
6. **Whisper download UI** (3.2) — Lets user download model from settings
7. **Dark mode toggle** (2.3) — Quick polish win
8. **Chat message persistence** (2.6) — Makes chat functional
9. **Empty states** (2.5) — Polish
10. **Timer display** (2.4) — Nice to have

---

## File Change Summary

| File | Changes |
|------|---------|
| `src-tauri/src/lib.rs` | Fix `stop_recording`, add `send_chat_message` command |
| `src-tauri/src/models.rs` | Add `ChatMessage` struct, add `messages` field to `ChatSession` |
| `src-tauri/src/pipeline.rs` | Ensure all events are emitted |
| `src-tauri/src/audio/recorder.rs` | Verify `stop()` thread safety |
| `src-tauri/src/ai/ollama.rs` | Verify streaming event emission |
| `src-tauri/tauri.conf.json` | Update app metadata |
| `src/lib/types.ts` | Add `ChatMessage` interface, update `ChatSession` |
| `src/lib/api.ts` | Add `sendChatMessage` API call |
| `src/components/ErrorBoundary.tsx` | New file |
| `src/components/layout/Sidebar.tsx` | Add dark mode toggle |
| `src/pages/ProcessingPage.tsx` | Fix navigation on complete |
| `src/pages/SettingsPage.tsx` | Add download/pull progress UI |
| `src/pages/HomePage.tsx` | Add setup banner |
| `src/pages/MeetingsPage.tsx` | Add empty state |
| `src/pages/ChatPage.tsx` | Add empty state, use messages |
| `src/pages/RecorderPage.tsx` | Add timer display |
| `src/App.tsx` | Wrap in ErrorBoundary |