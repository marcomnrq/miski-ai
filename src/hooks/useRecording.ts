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
  const { sessionName, notes } = useRecordingStore();

  const start = useCallback(
    async (name?: string) => {
      try {
        setError(null);
        const session = name || sessionName || `Meeting ${new Date().toLocaleDateString()}`;
        await api.recording.start(session);
        setState("recording");
        navigate("/recorder");

        // Start polling audio level every 100ms
        levelIntervalRef.current = window.setInterval(async () => {
          try {
            const level = await api.recording.getAudioLevel();
            setAudioLevel(level.rms);
            setDuration(level.duration_secs);
          } catch {
            // ignore polling errors
          }
        }, 100);
      } catch (e: unknown) {
        setError(e instanceof Error ? e.message : String(e));
      }
    },
    [navigate, sessionName]
  );

  const stop = useCallback(async () => {
    try {
      // Stop level polling
      if (levelIntervalRef.current) {
        clearInterval(levelIntervalRef.current);
        levelIntervalRef.current = null;
      }
      // Save notes before stopping
      if (notes) {
        await api.recording.updateNotes(notes);
      }

      setState("processing");
      setAudioLevel(0);
      navigate("/processing");

      await api.recording.stop();
      // The processing-complete event handler will update state
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
      setState("idle");
    }
  }, [navigate, notes]);

  const pause = useCallback(async () => {
    try {
      await api.recording.pause();
      setState("paused");
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  const resume = useCallback(async () => {
    try {
      await api.recording.resume();
      setState("recording");
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  // Listen for processing-complete
  useEffect(() => {
    const unlisten = events.onProcessingComplete((id) => {
      setMeetingId(id);
      setState("done");
      navigate(`/meetings/${id}`, { replace: true });
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