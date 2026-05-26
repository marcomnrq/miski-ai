import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Search, Clock, ArrowRight } from "lucide-react";
import { useMeetings } from "../hooks/useMeetings";

function formatDuration(seconds: number | null): string {
  if (!seconds) return "";
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m}m ${s}s`;
}

export function MeetingsPage() {
  const navigate = useNavigate();
  const { data: meetings, isLoading } = useMeetings();
  const [search, setSearch] = useState("");

  const filtered = (meetings ?? []).filter((m) => {
    if (!search.trim()) return true;
    const q = search.toLowerCase();
    return (
      m.title.toLowerCase().includes(q) ||
      (m.summary_preview ?? "").toLowerCase().includes(q)
    );
  });

  return (
    <div className="max-w-3xl mx-auto px-8 py-8">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-[var(--text-primary)]">
          Meetings
        </h1>
        <span className="text-sm text-[var(--text-secondary)]">
          {filtered.length} meeting{filtered.length !== 1 ? "s" : ""}
        </span>
      </div>

      {/* Search */}
      <div className="relative mb-6">
        <Search
          size={18}
          className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-secondary)]"
        />
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search meetings..."
          className="w-full bg-[var(--surface)] border border-[var(--border)] rounded-xl pl-10 pr-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-secondary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
        />
      </div>

      {/* List */}
      {isLoading ? (
        <p className="text-[var(--text-secondary)]">Loading...</p>
      ) : filtered.length === 0 ? (
        <div className="text-center py-16 bg-[var(--surface)] rounded-xl border border-[var(--border)]">
          <p className="text-[var(--text-secondary)]">
            {search ? "No meetings match your search." : "No meetings yet."}
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {filtered.map((m) => (
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
  );
}