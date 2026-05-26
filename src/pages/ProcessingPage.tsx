import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { AlertCircle } from "lucide-react";
import { events } from "../lib/api";
import { useStreaming } from "../hooks/useStreaming";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

export function ProcessingPage() {
  const [status, setStatus] = useState("Transcribing audio...");
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();
  const { text: summaryText, startStream } = useStreaming();

  useEffect(() => {
    // Listen for status updates
    const unlistenStatus = events.onProcessingStatus((s) => {
      setStatus(s);
      // Check if status is an error
      if (s.startsWith("Error:")) {
        setError(s);
      }
    });

    // Listen for processing complete → navigate to meeting
    const unlistenComplete = events.onProcessingComplete((meetingId) => {
      navigate(`/meetings/${meetingId}`, { replace: true });
    });

    // Start streaming summary
    startStream("summary-chunk", "summary-complete");

    return () => {
      unlistenStatus.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
    };
  }, [navigate, startStream]);

  // Error state
  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4 px-8">
        <AlertCircle size={48} className="text-[var(--danger)]" />
        <h2 className="text-xl font-bold text-[var(--text-primary)]">
          Processing Failed
        </h2>
        <p className="text-sm text-[var(--text-secondary)] max-w-md text-center">
          {error.replace("Error: ", "")}
        </p>
        <button
          onClick={() => navigate("/", { replace: true })}
          className="bg-[var(--accent)] text-white px-4 py-2 rounded-lg hover:bg-[var(--accent-hover)] transition-colors cursor-pointer"
        >
          Go Home
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center justify-center h-full gap-6 px-8">
      {/* Spinner */}
      <div className="w-12 h-12 border-4 border-[var(--border)] border-t-[var(--accent)] rounded-full animate-spin" />

      {/* Status */}
      <p className="text-lg text-[var(--text-primary)] font-medium">
        {status}
      </p>

      {/* Live summary preview */}
      {summaryText && (
        <div className="w-full max-w-2xl mt-4 p-6 bg-[var(--surface)] border border-[var(--border)] rounded-xl overflow-auto max-h-96">
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            className="prose prose-sm prose-invert max-w-none text-[var(--text-primary)]"
          >
            {summaryText}
          </ReactMarkdown>
        </div>
      )}
    </div>
  );
}
