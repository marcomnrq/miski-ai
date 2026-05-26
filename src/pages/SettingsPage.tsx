import { useState, useEffect } from "react";
import { Settings, CheckCircle, XCircle, AlertCircle, FolderOpen, Download, ExternalLink, Shield } from "lucide-react";
import { useSettings, useUpdateSettings } from "../hooks/useSettings";
import { useSetup } from "../hooks/useSetup";
import { api, events } from "../lib/api";
import type { SettingsPatch } from "../lib/types";
import { revealItemInDir } from "@tauri-apps/plugin-opener";

function StatusBadge({ ok, label }: { ok: boolean; label: string }) {
  return (
    <div className="flex items-center gap-2">
      {ok ? (
        <CheckCircle size={18} className="text-green-400" />
      ) : (
        <XCircle size={18} className="text-red-400" />
      )}
      <span className="text-sm text-[var(--text-primary)]">{label}</span>
      {!ok && (
        <span className="text-xs text-[var(--text-secondary)]">Not found</span>
      )}
    </div>
  );
}

export function SettingsPage() {
  const { data: settings, isLoading: settingsLoading } = useSettings();
  const { data: setup, isLoading: setupLoading } = useSetup();
  const updateSettings = useUpdateSettings();

  const [form, setForm] = useState<SettingsPatch>({});
  const [dataDir, setDataDir] = useState("");
  const [whisperDownloading, setWhisperDownloading] = useState(false);
  const [whisperProgress, setWhisperProgress] = useState(0);
  const [ollamaPulling, setOllamaPulling] = useState(false);
  const [ollamaPullProgress, setOllamaPullProgress] = useState(0);
  const [ollamaPullStatus, setOllamaPullStatus] = useState("");

  // Sync form when settings load
  useEffect(() => {
    if (settings) {
      setForm({
        whisper_model: settings.whisper_model,
        ollama_url: settings.ollama_url,
        ollama_model: settings.ollama_model,
        language: settings.language,
        ai_provider: settings.ai_provider,
        remote_ollama_url: settings.remote_ollama_url,
        keep_recordings: settings.keep_recordings,
        notifications_enabled: settings.notifications_enabled,
        user_name: settings.user_name ?? undefined,
      });
    }
  }, [settings]);

  // Load data directory
  useEffect(() => {
    api.settings.getDataDir().then(setDataDir).catch(() => {});
  }, []);

  // Listen for Whisper download progress
  useEffect(() => {
    const unlisten = events.onWhisperDownloadProgress((percent) => {
      setWhisperProgress(percent);
    });
    const unlistenComplete = events.onWhisperDownloadComplete(() => {
      setWhisperDownloading(false);
      setWhisperProgress(100);
    });
    return () => {
      unlisten.then((f) => f());
      unlistenComplete.then((f) => f());
    };
  }, []);

  // Listen for Ollama pull progress
  useEffect(() => {
    const unlisten = events.onModelPullProgress((progress) => {
      setOllamaPullProgress(progress.percent);
      setOllamaPullStatus(progress.status);
    });
    const unlistenComplete = events.onModelPullComplete(() => {
      setOllamaPulling(false);
      setOllamaPullProgress(100);
      setOllamaPullStatus("Complete");
    });
    return () => {
      unlisten.then((f) => f());
      unlistenComplete.then((f) => f());
    };
  }, []);

  const handleSave = () => {
    updateSettings.mutate(form);
  };

  const handleDownloadWhisper = () => {
    const model = form.whisper_model ?? "base";
    setWhisperDownloading(true);
    setWhisperProgress(0);
    api.setup.downloadWhisperModel(model).catch((e) => {
      console.error("Whisper download failed:", e);
      setWhisperDownloading(false);
    });
  };

  const handlePullOllama = () => {
    const model = form.ollama_model ?? "llama3.2:3b";
    setOllamaPulling(true);
    setOllamaPullProgress(0);
    setOllamaPullStatus("Starting...");
    api.setup.pullOllamaModel(model).catch((e) => {
      console.error("Ollama pull failed:", e);
      setOllamaPulling(false);
    });
  };

  const isLoading = settingsLoading || setupLoading;

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="w-8 h-8 border-3 border-[var(--border)] border-t-[var(--accent)] rounded-full animate-spin" />
      </div>
    );
  }

  return (
    <div className="max-w-2xl mx-auto px-8 py-8 space-y-8">
      <h1 className="text-2xl font-bold text-[var(--text-primary)] flex items-center gap-2">
        <Settings size={24} />
        Settings
      </h1>

      {/* Setup Status */}
      {setup && (
        <section className="bg-[var(--surface)] border border-[var(--border)] rounded-xl p-6 space-y-3">
          <h2 className="text-lg font-semibold text-[var(--text-primary)] mb-3">
            Setup Status
          </h2>
          <StatusBadge ok={setup.whisper_model_downloaded} label="Whisper Model" />
          <StatusBadge ok={setup.ollama_running} label="Ollama Running" />
          <StatusBadge ok={setup.ollama_model_pulled} label="Ollama Model" />
          <StatusBadge ok={setup.microphone_available} label="Microphone" />
          {!setup.whisper_model_downloaded && !whisperDownloading && (
            <div className="mt-3 p-3 bg-[var(--warning)]/10 border border-[var(--warning)]/30 rounded-lg">
              <div className="flex items-start gap-2">
                <AlertCircle size={18} className="text-[var(--warning)] shrink-0 mt-0.5" />
                <div className="text-sm text-[var(--text-primary)] flex-1">
                  <p className="font-medium">Whisper model not found</p>
                  <p className="text-[var(--text-secondary)] mt-1">
                    Download the selected model ({form.whisper_model ?? "base"}) automatically.
                  </p>
                </div>
                <button
                  onClick={handleDownloadWhisper}
                  className="flex items-center gap-1.5 px-3 py-1.5 bg-[var(--accent)] text-white rounded-lg text-xs font-medium hover:bg-[var(--accent-hover)] transition-colors shrink-0 cursor-pointer"
                >
                  <Download size={14} />
                  Download
                </button>
              </div>
            </div>
          )}
          {whisperDownloading && (
            <div className="mt-3 p-3 bg-[var(--surface-raised)] border border-[var(--border)] rounded-lg">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-[var(--text-primary)]">Downloading Whisper model...</span>
                <span className="text-xs text-[var(--text-secondary)]">{whisperProgress}%</span>
              </div>
              <div className="w-full h-2 bg-[var(--border)] rounded-full overflow-hidden">
                <div
                  className="h-full bg-[var(--accent)] rounded-full transition-all duration-300"
                  style={{ width: `${whisperProgress}%` }}
                />
              </div>
            </div>
          )}
          {!setup.ollama_running && (
            <div className="mt-3 p-3 bg-[var(--warning)]/10 border border-[var(--warning)]/30 rounded-lg flex items-start gap-2">
              <AlertCircle size={18} className="text-[var(--warning)] shrink-0 mt-0.5" />
              <div className="text-sm text-[var(--text-primary)]">
                <p className="font-medium">Ollama not running</p>
                <p className="text-[var(--text-secondary)] mt-1">
                  Start Ollama with:{" "}
                  <code className="bg-[var(--surface-raised)] px-1 rounded">
                    ollama serve
                  </code>
                </p>
              </div>
            </div>
          )}
          {setup.ollama_running && !setup.ollama_model_pulled && !ollamaPulling && (
            <div className="mt-3 p-3 bg-[var(--warning)]/10 border border-[var(--warning)]/30 rounded-lg">
              <div className="flex items-start gap-2">
                <AlertCircle size={18} className="text-[var(--warning)] shrink-0 mt-0.5" />
                <div className="text-sm text-[var(--text-primary)] flex-1">
                  <p className="font-medium">Ollama model not pulled</p>
                  <p className="text-[var(--text-secondary)] mt-1">
                    Pull {form.ollama_model ?? "llama3.2:3b"} to enable AI features.
                  </p>
                </div>
                <button
                  onClick={handlePullOllama}
                  className="flex items-center gap-1.5 px-3 py-1.5 bg-[var(--accent)] text-white rounded-lg text-xs font-medium hover:bg-[var(--accent-hover)] transition-colors shrink-0 cursor-pointer"
                >
                  <Download size={14} />
                  Pull Model
                </button>
              </div>
            </div>
          )}
          {ollamaPulling && (
            <div className="mt-3 p-3 bg-[var(--surface-raised)] border border-[var(--border)] rounded-lg">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-[var(--text-primary)]">Pulling {ollamaPullStatus}...</span>
                <span className="text-xs text-[var(--text-secondary)]">{ollamaPullProgress}%</span>
              </div>
              <div className="w-full h-2 bg-[var(--border)] rounded-full overflow-hidden">
                <div
                  className="h-full bg-[var(--accent)] rounded-full transition-all duration-300"
                  style={{ width: `${ollamaPullProgress}%` }}
                />
              </div>
            </div>
          )}
        </section>
      )}

      {/* Settings Form */}
      {settings && (
        <section className="bg-[var(--surface)] border border-[var(--border)] rounded-xl p-6 space-y-5">
          <h2 className="text-lg font-semibold text-[var(--text-primary)] mb-3">
            Configuration
          </h2>

          {/* Whisper Model */}
          <div>
            <label className="block text-sm font-medium text-[var(--text-primary)] mb-1">
              Whisper Model
            </label>
            <select
              value={form.whisper_model ?? "base"}
              onChange={(e) => setForm({ ...form, whisper_model: e.target.value })}
              className="w-full bg-[var(--background)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
            >
              <option value="tiny">Tiny</option>
              <option value="base">Base</option>
              <option value="small">Small</option>
              <option value="medium">Medium</option>
              <option value="large">Large</option>
            </select>
          </div>

          {/* Language */}
          <div>
            <label className="block text-sm font-medium text-[var(--text-primary)] mb-1">
              Language
            </label>
            <input
              type="text"
              value={form.language ?? "en"}
              onChange={(e) => setForm({ ...form, language: e.target.value })}
              className="w-full bg-[var(--background)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
              placeholder="en"
            />
          </div>

          {/* Ollama URL */}
          <div>
            <label className="block text-sm font-medium text-[var(--text-primary)] mb-1">
              Ollama URL
            </label>
            <input
              type="text"
              value={form.ollama_url ?? "http://localhost:11434"}
              onChange={(e) => setForm({ ...form, ollama_url: e.target.value })}
              className="w-full bg-[var(--background)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
              placeholder="http://localhost:11434"
            />
          </div>

          {/* Ollama Model */}
          <div>
            <label className="block text-sm font-medium text-[var(--text-primary)] mb-1">
              Ollama Model
            </label>
            <input
              type="text"
              value={form.ollama_model ?? "llama3"}
              onChange={(e) => setForm({ ...form, ollama_model: e.target.value })}
              className="w-full bg-[var(--background)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
              placeholder="llama3"
            />
          </div>

          {/* Keep Recordings toggle */}
          <div className="flex items-center justify-between">
            <label className="text-sm font-medium text-[var(--text-primary)]">
              Keep Recordings After Processing
            </label>
            <button
              onClick={() => setForm({ ...form, keep_recordings: !form.keep_recordings })}
              className={`relative w-11 h-6 rounded-full transition-colors cursor-pointer ${
                form.keep_recordings ? "bg-[var(--accent)]" : "bg-[var(--border)]"
              }`}
            >
              <div
                className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform ${
                  form.keep_recordings ? "translate-x-5" : ""
                }`}
              />
            </button>
          </div>

          {/* Notifications toggle */}
          <div className="flex items-center justify-between">
            <label className="text-sm font-medium text-[var(--text-primary)]">
              Notifications
            </label>
            <button
              onClick={() => setForm({ ...form, notifications_enabled: !form.notifications_enabled })}
              className={`relative w-11 h-6 rounded-full transition-colors cursor-pointer ${
                form.notifications_enabled ? "bg-[var(--accent)]" : "bg-[var(--border)]"
              }`}
            >
              <div
                className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform ${
                  form.notifications_enabled ? "translate-x-5" : ""
                }`}
              />
            </button>
          </div>

          {/* User Name */}
          <div>
            <label className="block text-sm font-medium text-[var(--text-primary)] mb-1">
              Your Name (optional)
            </label>
            <input
              type="text"
              value={form.user_name ?? ""}
              onChange={(e) => setForm({ ...form, user_name: e.target.value || undefined })}
              className="w-full bg-[var(--background)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
              placeholder="Your name"
            />
          </div>

          {/* Save button */}
          <button
            onClick={handleSave}
            disabled={updateSettings.isPending}
            className="bg-[var(--accent)] text-white px-6 py-2.5 rounded-lg font-medium hover:bg-[var(--accent-hover)] transition-colors disabled:opacity-50 cursor-pointer"
          >
            {updateSettings.isPending ? "Saving..." : "Save Settings"}
          </button>

          {updateSettings.isSuccess && (
            <p className="text-green-400 text-sm">Settings saved!</p>
          )}
          {updateSettings.isError && (
            <p className="text-[var(--danger)] text-sm">Failed to save settings.</p>
          )}
        </section>
      )}

      {/* Data Directory */}
      {dataDir && (
        <section className="bg-[var(--surface)] border border-[var(--border)] rounded-xl p-6">
          <div className="flex items-center gap-2 mb-2">
            <FolderOpen size={18} className="text-[var(--text-secondary)]" />
            <h2 className="text-lg font-semibold text-[var(--text-primary)]">
              Data Directory
            </h2>
          </div>
          <p className="text-sm text-[var(--text-secondary)] font-mono break-all mb-3">
            {dataDir}
          </p>
          <div className="flex flex-col gap-3">
            <button
              onClick={() => revealItemInDir(dataDir)}
              className="flex items-center gap-2 text-sm text-[var(--accent)] hover:underline cursor-pointer w-fit"
            >
              <ExternalLink size={14} />
              Open in Finder
            </button>
            <div className="text-xs text-[var(--text-secondary)] space-y-1.5 bg-[var(--surface-raised)] rounded-lg p-3">
              <p>
                This folder contains your <strong>settings</strong>, <strong>meeting transcripts</strong>,{" "}
                <strong>AI summaries</strong>, <strong>chat history</strong>, <strong>audio recordings</strong>,{" "}
                and <strong>Whisper model weights</strong>.
              </p>
              <div className="flex items-start gap-1.5 text-green-400">
                <Shield size={14} className="shrink-0 mt-0.5" />
                <p>
                  Everything stays on your Mac. <strong>Nothing is uploaded or sent to any server.</strong>{" "}
                  All AI processing happens locally through Ollama.
                </p>
              </div>
            </div>
          </div>
        </section>
      )}
    </div>
  );
}