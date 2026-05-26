import { useState, useCallback, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export function useStreaming() {
  const [text, setText] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const unlisteners = useRef<UnlistenFn[]>([]);

  const startStream = useCallback(
    async (
      chunkEvent: "summary-chunk" | "query-chunk",
      doneEvent: "summary-complete" | "query-complete"
    ) => {
      setText("");
      setIsStreaming(true);

      const unlistenChunk = await listen<string>(chunkEvent, (e) => {
        setText((prev) => prev + e.payload);
      });

      const unlistenDone = await listen<void>(doneEvent, () => {
        setIsStreaming(false);
        // Cleanup listeners
        unlistenChunk();
        unlistenDone();
        unlisteners.current = unlisteners.current.filter(
          (fn) => fn !== unlistenChunk && fn !== unlistenDone
        );
      });

      unlisteners.current.push(unlistenChunk, unlistenDone);
    },
    []
  );

  const reset = useCallback(() => {
    setText("");
    setIsStreaming(false);
    unlisteners.current.forEach((fn) => fn());
    unlisteners.current = [];
  }, []);

  return { text, isStreaming, startStream, reset };
}