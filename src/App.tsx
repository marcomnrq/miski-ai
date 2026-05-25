import { useState } from "react";

// ── Sidebar Navigation ──────────────────────────────────────────

type Route = "home" | "recorder" | "meetings" | "chat" | "settings";

const NAV_ITEMS: { id: Route; label: string; icon: string }[] = [
  { id: "home", label: "Home", icon: "🏠" },
  { id: "recorder", label: "Record", icon: "🎙️" },
  { id: "meetings", label: "Meetings", icon: "📝" },
  { id: "chat", label: "Chat", icon: "💬" },
  { id: "settings", label: "Settings", icon: "⚙️" },
];

// ── Page Placeholders ───────────────────────────────────────────

function HomePage() {
  return (
    <div className="flex flex-col items-center justify-center h-full gap-6">
      <h1 className="text-3xl font-semibold text-[var(--text-primary)]">
        Welcome to Miski AI
      </h1>
      <p className="text-[var(--text-secondary)] text-center max-w-md">
        Your private, on-device meeting recorder with transcription, AI
        summarization, and speaker diarisation. Press Record to get started.
      </p>
      <button className="bg-[var(--accent)] text-white px-6 py-3 rounded-xl font-medium hover:bg-[var(--accent-hover)] transition-colors">
        Start Recording
      </button>
    </div>
  );
}

function RecorderPage() {
  return (
    <div className="flex flex-col items-center justify-center h-full gap-4">
      <div className="text-6xl">🎙️</div>
      <p className="text-[var(--text-secondary)]">
        Recording will appear here — cpal integration pending.
      </p>
    </div>
  );
}

function MeetingsPage() {
  return (
    <div className="flex flex-col items-center justify-center h-full gap-4">
      <div className="text-6xl">📝</div>
      <p className="text-[var(--text-secondary)]">
        Your meetings will appear here — SQLite integration pending.
      </p>
    </div>
  );
}

function ChatPage() {
  return (
    <div className="flex flex-col items-center justify-center h-full gap-4">
      <div className="text-6xl">💬</div>
      <p className="text-[var(--text-secondary)]">
        AI Q&A will appear here — Ollama integration pending.
      </p>
    </div>
  );
}

function SettingsPage() {
  return (
    <div className="flex flex-col items-center justify-center h-full gap-4">
      <div className="text-6xl">⚙️</div>
      <p className="text-[var(--text-secondary)]">
        Settings will appear here — rusqlite integration pending.
      </p>
    </div>
  );
}

// ── Main App ────────────────────────────────────────────────────

function App() {
  const [route, setRoute] = useState<Route>("home");

  const renderPage = () => {
    switch (route) {
      case "home":
        return <HomePage />;
      case "recorder":
        return <RecorderPage />;
      case "meetings":
        return <MeetingsPage />;
      case "chat":
        return <ChatPage />;
      case "settings":
        return <SettingsPage />;
    }
  };

  return (
    <div className="flex h-screen bg-[var(--background)] text-[var(--text-primary)]">
      {/* Sidebar */}
      <aside className="w-56 border-r border-[var(--border)] bg-[var(--surface)] flex flex-col">
        <div className="p-4 border-b border-[var(--border)]">
          <h2 className="text-lg font-semibold">Miski AI</h2>
          <p className="text-xs text-[var(--text-secondary)]">
            Private meeting notes
          </p>
        </div>

        <nav className="flex-1 p-2">
          {NAV_ITEMS.map((item) => (
            <button
              key={item.id}
              onClick={() => setRoute(item.id)}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${
                route === item.id
                  ? "bg-[var(--accent)] text-white"
                  : "text-[var(--text-secondary)] hover:bg-[var(--surface-raised)]"
              }`}
            >
              <span>{item.icon}</span>
              <span>{item.label}</span>
            </button>
          ))}
        </nav>

        <div className="p-3 border-t border-[var(--border)]">
          <p className="text-xs text-[var(--text-secondary)]">
            Powered by Rust + React
          </p>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-auto">{renderPage()}</main>
    </div>
  );
}

export default App;
