import { useNavigate, Link } from "react-router-dom";
import { Mic, Clock, ArrowRight, AlertTriangle } from "lucide-react";
import { useMeetings } from "../hooks/useMeetings";
import { useRecording } from "../hooks/useRecording";
import { useSetup } from "../hooks/useSetup";

function formatDuration(seconds: number | null): string {
  if (!seconds) return "";
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}

export function HomePage() {
  const navigate = useNavigate();
  const { data: meetings, isLoading } = useMeetings();
  const { data: setup } = useSetup();
  const { start } = useRecording();

  const recentMeetings = meetings?.slice(0, 5) ?? [];

  const needsSetup = setup && (
    !setup.whisper_model_downloaded ||
    !setup.ollama_running ||
    !setup.ollama_model_pulled
  );

  return (
    <div className="max-w-3xl mx-auto px-8 py-12">
      {/* Setup Banner */}
      {needsSetup && (
        <div className="bg-[var(--warning)]/10 border border-[var(--warning)]/30 rounded-xl p-4 mb-6 flex items-center gap-3">
          <AlertTriangle size={20} className="text-[var(--warning)] shrink-0" />
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

      {/* Hero */}
      <div className="text-center mb-12">
        <h1 className="text-4xl font-bold text-[var(--text-primary)] mb-3">
          Miski AI
        </h1>
        <p className="text-[var(--text-secondary)] text-lg mb-8">
          Privacy-first meeting recorder with AI transcription & summarization
        </p>
        <button
          onClick={() => start()}
          className="inline-flex items-center gap-2 bg-[var(--accent)] text-white px-8 py-4 rounded-xl font-semibold text-lg hover:bg-[var(--accent-hover)] transition-colors cursor-pointer"
        >
          <Mic size={22} />
          Start Recording
        </button>
      </div>

      {/* Recent Meetings */}
      <div>
        <h2 className="text-xl font-semibold text-[var(--text-primary)] mb-4">
          Recent Meetings
        </h2>
        {isLoading ? (
          <p className="text-[var(--text-secondary)]">Loading...</p>
        ) : recentMeetings.length === 0 ? (
          <div className="text-center py-12 bg-[var(--surface)] rounded-xl border border-[var(--border)]">
            <p className="text-[var(--text-secondary)]">
              No meetings yet. Start recording to create your first one!
            </p>
          </div>
        ) : (
          <div className="space-y-3">
            {recentMeetings.map((m) => (
              <button
                key={m.id}
                onClick={() => navigate(`/meetings/${m.id}`)}
                className="w-full text-left p-4 bg-[var(--surface)] border border-[var(--border)] rounded-xl hover:border-[var(--accent)] transition-colors group cursor-pointer"
              >
                <div className="flex items-center justify-between">
                  <h3 className="font-medium text-[var(--text-primary)] group-hover:text-[var(--accent)] transition-colors">
                    {m.title}
                  </h3>
                  <ArrowRight
                    size={16}
                    className="text-[var(--text-secondary)] opacity-0 group-hover:opacity-100 transition-opacity"
                  />
                </div>
                <div className="flex items-center gap-4 mt-1 text-sm text-[var(--text-secondary)]">
                  <span>{new Date(m.created_at).toLocaleDateString()}</span>
                  {m.duration_seconds != null && (
                    <span className="flex items-center gap-1">
                      <Clock size={14} />
                      {formatDuration(m.duration_seconds)}
                    </span>
                  )}
                  <span className="px-2 py-0.5 rounded bg-[var(--surface-raised)] text-xs">
                    {m.language}
                  </span>
                  {m.is_diarised && (
                    <span className="px-2 py-0.5 rounded bg-[var(--accent)]/10 text-[var(--accent)] text-xs">
                      Diarised
                    </span>
                  )}
                </div>
                {m.summary_preview && (
                  <p className="mt-2 text-sm text-[var(--text-secondary)] line-clamp-2">
                    {m.summary_preview}
                  </p>
                )}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}