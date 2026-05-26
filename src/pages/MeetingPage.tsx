import { useState, useEffect, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Clock, Download, Trash2, Send, Globe } from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { useMeeting, useRenameMeeting, useSaveNotes, useDeleteMeeting } from "../hooks/useMeetings";
import { useStreaming } from "../hooks/useStreaming";
import { api } from "../lib/api";

// Speaker color palette — cycles through these for each unique speaker
const SPEAKER_COLORS = [
  "bg-blue-500/20 text-blue-400",
  "bg-green-500/20 text-green-400",
  "bg-purple-500/20 text-purple-400",
  "bg-orange-500/20 text-orange-400",
  "bg-pink-500/20 text-pink-400",
  "bg-cyan-500/20 text-cyan-400",
];

function formatDuration(seconds: number | null): string {
  if (!seconds) return "Unknown";
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m}m ${s}s`;
}

/** Parse diarised text format: `[Speaker A] text` lines */
function parseDiarisedText(text: string): { speaker: string; text: string }[] {
  const lines = text.split("\n\n").filter(Boolean);
  return lines.map((line) => {
    const match = line.match(/^\[([^\]]+)\]\s*(.*)/s);
    if (match) {
      return { speaker: match[1], text: match[2].trim() };
    }
    return { speaker: "Speaker", text: line.trim() };
  });
}

type Tab = "summary" | "transcript" | "notes";

export function MeetingPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: meeting, isLoading } = useMeeting(id ?? "");
  const renameMeeting = useRenameMeeting();
  const saveNotes = useSaveNotes();
  const deleteMeeting = useDeleteMeeting();

  const [activeTab, setActiveTab] = useState<Tab>("summary");
  const [editingTitle, setEditingTitle] = useState(false);
  const [titleDraft, setTitleDraft] = useState("");
  const [notesDraft, setNotesDraft] = useState("");
  const [question, setQuestion] = useState("");
  const { text: answerText, isStreaming, startStream, reset: resetStream } = useStreaming();

  // Sync notes draft when meeting loads
  useEffect(() => {
    if (meeting?.notes != null) {
      setNotesDraft(meeting.notes);
    }
  }, [meeting?.notes]);

  // Save title on blur
  const handleTitleSave = useCallback(() => {
    if (id && titleDraft.trim() && titleDraft !== meeting?.title) {
      renameMeeting.mutate({ id, name: titleDraft.trim() });
    }
    setEditingTitle(false);
  }, [id, titleDraft, meeting?.title, renameMeeting]);

  // Save notes on blur
  const handleNotesSave = useCallback(() => {
    if (id && notesDraft !== (meeting?.notes ?? "")) {
      saveNotes.mutate({ id, notes: notesDraft });
    }
  }, [id, notesDraft, meeting?.notes, saveNotes]);

  // Ask a question
  const handleAsk = useCallback(async () => {
    if (!id || !question.trim() || isStreaming) return;
    resetStream();
    const q = question.trim();
    setQuestion("");
    await api.meetings.query(id, q);
    startStream("query-chunk", "query-complete");
  }, [id, question, isStreaming, resetStream, startStream]);

  // Delete meeting
  const handleDelete = useCallback(() => {
    if (!id) return;
    if (confirm("Delete this meeting? This cannot be undone.")) {
      deleteMeeting.mutate(id, {
        onSuccess: () => navigate("/meetings"),
      });
    }
  }, [id, deleteMeeting, navigate]);

  // Export markdown
  const handleExport = useCallback(async () => {
    if (!id) return;
    try {
      const md = await api.meetings.exportMarkdown(id);
      // Copy to clipboard as a simple export
      await navigator.clipboard.writeText(md);
      alert("Markdown copied to clipboard!");
    } catch {
      alert("Failed to export markdown.");
    }
  }, [id]);

  if (isLoading || !meeting) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="w-8 h-8 border-3 border-[var(--border)] border-t-[var(--accent)] rounded-full animate-spin" />
      </div>
    );
  }

  const tabs: { key: Tab; label: string }[] = [
    { key: "summary", label: "Summary" },
    { key: "transcript", label: "Transcript" },
    { key: "notes", label: "Notes" },
  ];

  // Build speaker color map
  const speakerColorMap = new Map<string, string>();
  if (meeting.diarised_text) {
    const segments = parseDiarisedText(meeting.diarised_text);
    let colorIdx = 0;
    for (const seg of segments) {
      if (!speakerColorMap.has(seg.speaker)) {
        speakerColorMap.set(seg.speaker, SPEAKER_COLORS[colorIdx % SPEAKER_COLORS.length]);
        colorIdx++;
      }
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="px-8 pt-6 pb-4 border-b border-[var(--border)]">
        <div className="flex items-start justify-between">
          <div className="flex-1">
            {editingTitle ? (
              <input
                type="text"
                value={titleDraft}
                onChange={(e) => setTitleDraft(e.target.value)}
                onBlur={handleTitleSave}
                onKeyDown={(e) => e.key === "Enter" && handleTitleSave()}
                autoFocus
                className="text-2xl font-bold bg-transparent border-b-2 border-[var(--accent)] outline-none text-[var(--text-primary)] w-full"
              />
            ) : (
              <h1
                onClick={() => {
                  setTitleDraft(meeting.title);
                  setEditingTitle(true);
                }}
                className="text-2xl font-bold text-[var(--text-primary)] cursor-pointer hover:text-[var(--accent)] transition-colors"
              >
                {meeting.title}
              </h1>
            )}
            <div className="flex items-center gap-4 mt-2 text-sm text-[var(--text-secondary)]">
              <span>{new Date(meeting.created_at).toLocaleDateString()}</span>
              <span className="flex items-center gap-1">
                <Clock size={14} />
                {formatDuration(meeting.duration_seconds ?? null)}
              </span>
              <span className="flex items-center gap-1">
                <Globe size={14} />
                {meeting.language}
              </span>
              {meeting.whisper_model && (
                <span className="px-2 py-0.5 rounded bg-[var(--surface-raised)] text-xs">
                  {meeting.whisper_model}
                </span>
              )}
              {meeting.is_diarised && (
                <span className="px-2 py-0.5 rounded bg-[var(--accent)]/10 text-[var(--accent)] text-xs">
                  Diarised
                </span>
              )}
            </div>
          </div>

          {/* Action buttons */}
          <div className="flex items-center gap-2 ml-4">
            <button
              onClick={handleExport}
              className="p-2 rounded-lg hover:bg-[var(--surface)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors cursor-pointer"
              title="Export Markdown"
            >
              <Download size={18} />
            </button>
            <button
              onClick={handleDelete}
              className="p-2 rounded-lg hover:bg-[var(--danger)]/10 text-[var(--text-secondary)] hover:text-[var(--danger)] transition-colors cursor-pointer"
              title="Delete meeting"
            >
              <Trash2 size={18} />
            </button>
          </div>
        </div>

        {/* Tabs */}
        <div className="flex gap-1 mt-4">
          {tabs.map(({ key, label }) => (
            <button
              key={key}
              onClick={() => setActiveTab(key)}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors cursor-pointer ${
                activeTab === key
                  ? "bg-[var(--accent)] text-white"
                  : "text-[var(--text-secondary)] hover:bg-[var(--surface)] hover:text-[var(--text-primary)]"
              }`}
            >
              {label}
            </button>
          ))}
        </div>
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-auto px-8 py-6">
        {/* Summary Tab */}
        {activeTab === "summary" && (
          <div className="max-w-3xl">
            {meeting.summary_markdown ? (
              <ReactMarkdown
                remarkPlugins={[remarkGfm]}
                className="prose prose-sm prose-invert max-w-none text-[var(--text-primary)]"
              >
                {meeting.summary_markdown}
              </ReactMarkdown>
            ) : (
              <p className="text-[var(--text-secondary)]">No summary available.</p>
            )}
          </div>
        )}

        {/* Transcript Tab */}
        {activeTab === "transcript" && (
          <div className="max-w-3xl space-y-3">
            {meeting.is_diarised && meeting.diarised_text ? (
              parseDiarisedText(meeting.diarised_text).map((seg, i) => (
                <div key={i} className="flex gap-3">
                  <span
                    className={`shrink-0 px-2 py-1 rounded text-xs font-medium self-start mt-0.5 ${
                      speakerColorMap.get(seg.speaker) ?? "bg-[var(--surface-raised)] text-[var(--text-secondary)]"
                    }`}
                  >
                    {seg.speaker}
                  </span>
                  <p className="text-[var(--text-primary)] text-sm leading-relaxed">
                    {seg.text}
                  </p>
                </div>
              ))
            ) : meeting.transcript_text ? (
              <p className="text-[var(--text-primary)] text-sm leading-relaxed whitespace-pre-wrap">
                {meeting.transcript_text}
              </p>
            ) : (
              <p className="text-[var(--text-secondary)]">No transcript available.</p>
            )}
          </div>
        )}

        {/* Notes Tab */}
        {activeTab === "notes" && (
          <div className="max-w-3xl">
            <textarea
              value={notesDraft}
              onChange={(e) => setNotesDraft(e.target.value)}
              onBlur={handleNotesSave}
              placeholder="Add notes about this meeting..."
              className="w-full min-h-[300px] bg-[var(--surface)] border border-[var(--border)] rounded-xl p-4 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-secondary)] resize-y focus:outline-none focus:border-[var(--accent)] transition-colors"
            />
          </div>
        )}

        {/* Q&A Answer */}
        {answerText && (
          <div className="max-w-3xl mt-6 p-4 bg-[var(--surface)] border border-[var(--border)] rounded-xl">
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              className="prose prose-sm prose-invert max-w-none text-[var(--text-primary)]"
            >
              {answerText}
            </ReactMarkdown>
          </div>
        )}
      </div>

      {/* AskBar */}
      <div className="px-8 py-4 border-t border-[var(--border)] bg-[var(--surface)]">
        <div className="flex items-center gap-3 max-w-3xl mx-auto">
          <input
            type="text"
            value={question}
            onChange={(e) => setQuestion(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAsk()}
            placeholder="Ask a question about this meeting..."
            className="flex-1 bg-[var(--background)] border border-[var(--border)] rounded-xl px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-secondary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
            disabled={isStreaming}
          />
          <button
            onClick={handleAsk}
            disabled={isStreaming || !question.trim()}
            className="p-2.5 rounded-xl bg-[var(--accent)] text-white hover:bg-[var(--accent-hover)] transition-colors disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
          >
            <Send size={18} />
          </button>
        </div>
      </div>
    </div>
  );
}