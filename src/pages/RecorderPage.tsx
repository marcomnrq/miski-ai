import { useState } from "react";
import { Pause, Play, Square, ChevronDown, ChevronUp, Mic } from "lucide-react";
import { useRecording } from "../hooks/useRecording";
import { useRecordingStore } from "../stores/recordingStore";

function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

export function RecorderPage() {
  const { state, audioLevel, duration, error, start, stop, pause, resume } =
    useRecording();
  const { sessionName, notes, setSessionName, setNotes } =
    useRecordingStore();
  const [showNotes, setShowNotes] = useState(false);

  // Idle state: show start button
  if (state === "idle") {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-6">
        <div className="w-20 h-20 rounded-full bg-[var(--surface)] border border-[var(--border)] flex items-center justify-center">
          <Mic size={36} className="text-[var(--text-secondary)]" />
        </div>
        <p className="text-[var(--text-secondary)] text-lg">
          Ready to record
        </p>
        <button
          onClick={() => start()}
          className="bg-[var(--accent)] text-white px-8 py-4 rounded-xl font-semibold text-lg hover:bg-[var(--accent-hover)] transition-colors cursor-pointer"
        >
          Start Recording
        </button>
        {error && (
          <p className="text-[var(--danger)] text-sm bg-[var(--danger)]/10 px-4 py-2 rounded-lg">
            {error}
          </p>
        )}
      </div>
    );
  }

  // Recording or Paused state
  const isPaused = state === "paused";

  return (
    <div className="max-w-2xl mx-auto px-8 py-12 flex flex-col items-center gap-8">
      {/* Session name */}
      <input
        type="text"
        value={sessionName}
        onChange={(e) => setSessionName(e.target.value)}
        className="text-xl font-semibold text-center bg-transparent border-b-2 border-transparent hover:border-[var(--border)] focus:border-[var(--accent)] outline-none px-4 py-1 text-[var(--text-primary)] transition-colors w-full max-w-md"
        placeholder="Meeting name..."
      />

      {/* Timer */}
      <div className="font-mono text-6xl tabular-nums text-[var(--text-primary)]">
        {formatTime(duration)}
      </div>

      {/* Audio waveform */}
      <div className="flex items-end gap-1 h-16">
        {Array.from({ length: 8 }).map((_, i) => {
          const baseHeight = 8;
          const variableHeight = audioLevel * 48;
          // Stagger bars for visual interest using a sine curve
          const stagger = Math.sin((i / 7) * Math.PI) * 0.7 + 0.3;
          const height = baseHeight + variableHeight * stagger;
          return (
            <div
              key={i}
              className={`w-3 rounded-full transition-all duration-100 ${
                isPaused
                  ? "bg-[var(--border)]"
                  : "bg-[var(--accent)]"
              }`}
              style={{ height: `${Math.max(4, height)}px` }}
            />
          );
        })}
      </div>

      {/* Status indicator */}
      <div className="flex items-center gap-2">
        <div
          className={`w-3 h-3 rounded-full ${
            isPaused
              ? "bg-[var(--warning)]"
              : "bg-[var(--danger)] animate-pulse"
          }`}
        />
        <span className="text-sm text-[var(--text-secondary)]">
          {isPaused ? "Paused" : "Recording"}
        </span>
      </div>

      {/* Control buttons */}
      <div className="flex items-center gap-4">
        <button
          onClick={() => (isPaused ? resume() : pause())}
          className="p-4 rounded-full bg-[var(--surface)] border border-[var(--border)] hover:bg-[var(--surface-raised)] transition-colors text-[var(--text-primary)] cursor-pointer"
          title={isPaused ? "Resume" : "Pause"}
        >
          {isPaused ? <Play size={24} /> : <Pause size={24} />}
        </button>
        <button
          onClick={stop}
          className="p-4 rounded-full bg-[var(--danger)] text-white hover:opacity-90 transition-opacity cursor-pointer"
          title="Stop"
        >
          <Square size={24} fill="currentColor" />
        </button>
      </div>

      {/* Notes section */}
      <div className="w-full max-w-md">
        <button
          onClick={() => setShowNotes(!showNotes)}
          className="flex items-center gap-2 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors w-full justify-center cursor-pointer"
        >
          Notes
          {showNotes ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
        </button>
        {showNotes && (
          <textarea
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
            placeholder="Add notes during the meeting..."
            className="mt-2 w-full h-32 bg-[var(--surface)] border border-[var(--border)] rounded-xl p-3 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-secondary)] resize-none focus:outline-none focus:border-[var(--accent)] transition-colors"
          />
        )}
      </div>

      {error && (
        <p className="text-[var(--danger)] text-sm bg-[var(--danger)]/10 px-4 py-2 rounded-lg">
          {error}
        </p>
      )}
    </div>
  );
}