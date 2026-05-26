<div align="center">
  <img src="public/logo-rounded.png" alt="Miski AI Logo" width="128" height="128" />
  
  # Miski AI
  
  > Privacy-first, on-device meeting recorder with transcription, AI summarization, and speaker diarisation.
</div>

Miski AI is a complete rewrite of [StenoAI](https://github.com/ruzin/stenoai), replacing the Electron + Python stack with a single **Tauri v2 + Rust + React** application. Same core features, dramatically simpler architecture.

## Why Miski AI?

| | StenoAI | Miski AI |
|---|---|---|
| Desktop shell | Electron (~200MB) | Tauri (~15MB) |
| Backend | Python (~2.8K lines CLI) | Rust (~1.5K lines) |
| IPC mechanism | stdout parsing, base64 chunks | Tauri typed commands + events |
| Data storage | JSON + Markdown files | JSON files + Markdown export |
| Build system | PyInstaller + electron-builder | `cargo tauri build` |
| Total lines | ~12,500 | ~4,850 (est.) |

## Features

- **Microphone recording** — Native audio capture via `cpal` (CoreAudio)
- **On-device transcription** — whisper.cpp via `whisper-rs`, no internet required
- **Speaker diarisation** — Silence-gap turn detection + LLM speaker labeling
- **AI summarization** — Ollama HTTP client with streaming responses
- **99 languages** — Auto-detect and transcribe via whisper.cpp
- **In-app note-taking** — Jot notes during recording, folded into AI summary
- **Chat / Q&A** — Natural language queries across your meetings
- **Markdown export** — Clean, portable notes for sharing and searching
- **Privacy-first** — 100% on-device. Your data never leaves your Mac

## Tech Stack

```
Frontend:   React 19 · TypeScript · Vite · Tailwind CSS v4
Shell:      Tauri v2 (WKWebView, ~15MB binary)
Backend:    Rust (cpal, whisper-rs, reqwest, tokio)
AI:         Ollama (local or remote)
Storage:    JSON files in ~/Library/Application Support/miski-ai/
```

## Project Structure

```
miski-ai/
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs               # Tauri entry point
│       ├── lib.rs                # Command registration + AppState
│       ├── models.rs             # Shared serde types
│       ├── commands/             # Tauri command handlers
│       │   ├── mod.rs
│       │   ├── recording.rs
│       │   ├── transcription.rs
│       │   ├── summarization.rs
│       │   ├── meetings.rs
│       │   ├── chat.rs
│       │   ├── settings.rs
│       │   └── setup.rs
│       ├── audio/                # Audio capture
│       │   ├── mod.rs
│       │   └── recorder.rs
│       ├── transcription/        # Transcription pipeline
│       │   ├── mod.rs
│       │   ├── whisper.rs
│       │   └── diarisation.rs
│       ├── ai/                   # LLM integration
│       │   ├── mod.rs
│       │   ├── ollama.rs
│       │   └── prompts.rs
│       └── storage/              # JSON persistence
│           ├── mod.rs
│           ├── json_store.rs
│           └── markdown.rs
├── src/                          # React frontend
│   ├── main.tsx
│   ├── App.tsx
│   ├── globals.css
│   ├── lib/
│   │   └── api.ts
│   ├── hooks/
│   ├── components/
│   │   ├── ui/
│   │   ├── layout/
│   │   ├── recorder/
│   │   ├── meetings/
│   │   ├── chat/
│   │   └── setup/
│   ├── pages/
│   └── routes/
│       └── index.tsx
└── package.json
```

## Getting Started

### Prerequisites

- **macOS 14 Sonoma** or later
- **Rust** — [rustup.rs](https://rustup.rs) or `brew install rustup-init && rustup-init`
- **Node.js 18+** — `brew install node`
- **pnpm** — `brew install pnpm` (or `npm install -g pnpm`)
- **Ollama** — `brew install ollama`

### Development

```bash
# Install frontend dependencies
cd miski-ai
pnpm install

# Start dev server (hot reload for frontend, auto-rebuild for Rust)
pnpm tauri dev
```

### Production Build

```bash
pnpm tauri build
# Output: target/release/bundle/dmg/Miski AI_0.1.0_aarch64.dmg
```

## Data Storage

All data is stored locally in `~/Library/Application Support/miski-ai/`:

```
miski-ai/
├── config.json           # App settings
├── meetings/             # One JSON file per meeting
│   └── {uuid}.json
├── chat/                 # One JSON file per chat session
│   └── {uuid}.json
├── recordings/           # Temporary WAV files
└── whisper_models/       # Whisper model weights
```

## License

MIT
