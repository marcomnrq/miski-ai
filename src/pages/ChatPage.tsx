import { useState, useCallback, useRef, useEffect } from "react";
import { Send, MessageSquare, Plus, Trash2 } from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { useStreaming } from "../hooks/useStreaming";
import { api } from "../lib/api";
import type { ChatSession, ChatMessage } from "../lib/types";

export function ChatPage() {
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(true);
  const { text: streamText, isStreaming, startStream, reset: resetStream } = useStreaming();
  const scrollRef = useRef<HTMLDivElement>(null);

  // Active session
  const activeSession = sessions.find((s) => s.id === activeSessionId) ?? null;

  // Messages from the active session + live streaming text
  const messages = activeSession?.messages ?? [];

  // Load sessions on mount
  useEffect(() => {
    api.chat.listSessions().then((s) => {
      setSessions(s);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, []);

  // Auto-scroll on new content
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages, streamText]);

  // When streaming completes, reload the session to get persisted messages
  useEffect(() => {
    if (!isStreaming && streamText && activeSessionId) {
      api.chat.getSession(activeSessionId).then((updated) => {
        setSessions((prev) =>
          prev.map((s) => (s.id === updated.id ? updated : s))
        );
      });
      resetStream();
    }
  }, [isStreaming, streamText, activeSessionId, resetStream]);

  const handleNewSession = useCallback(async () => {
    try {
      const session = await api.chat.createSession("all");
      setSessions((prev) => [session, ...prev]);
      setActiveSessionId(session.id);
      resetStream();
    } catch (e) {
      console.error("Failed to create session:", e);
    }
  }, [resetStream]);

  const handleDeleteSession = useCallback(
    async (id: string) => {
      try {
        await api.chat.deleteSession(id);
        setSessions((prev) => prev.filter((s) => s.id !== id));
        if (activeSessionId === id) {
          setActiveSessionId(null);
        }
      } catch (e) {
        console.error("Failed to delete session:", e);
      }
    },
    [activeSessionId]
  );

  const handleSend = useCallback(async () => {
    const q = input.trim();
    if (!q || isStreaming) return;

    // Create a session if none is active
    let sessionId = activeSessionId;
    if (!sessionId) {
      try {
        const session = await api.chat.createSession("all");
        setSessions((prev) => [session, ...prev]);
        sessionId = session.id;
        setActiveSessionId(session.id);
      } catch (e) {
        console.error("Failed to create session:", e);
        return;
      }
    }

    setInput("");

    // Optimistically add user message to UI
    const tempUserMsg: ChatMessage = {
      id: `temp-${Date.now()}`,
      session_id: sessionId,
      role: "user",
      content: q,
      created_at: new Date().toISOString(),
    };
    setSessions((prev) =>
      prev.map((s) =>
        s.id === sessionId ? { ...s, messages: [...s.messages, tempUserMsg] } : s
      )
    );

    // Start streaming listener before sending
    resetStream();
    startStream("query-chunk", "query-complete");

    // Send message (persists user msg, streams AI response, persists assistant msg)
    try {
      await api.chat.sendMessage(sessionId, q);
    } catch (e) {
      console.error("Failed to send message:", e);
    }
  }, [input, isStreaming, activeSessionId, resetStream, startStream]);

  // Build display messages: persisted + live streaming assistant text
  const displayMessages = [...messages];
  if (isStreaming && streamText) {
    const lastMsg = displayMessages[displayMessages.length - 1];
    if (lastMsg?.role === "assistant" && lastMsg.content === "") {
      // Replace empty assistant placeholder with stream text
      displayMessages[displayMessages.length - 1] = {
        ...lastMsg,
        content: streamText,
      };
    } else {
      // Append streaming assistant message
      displayMessages.push({
        id: "streaming",
        session_id: activeSessionId ?? "",
        role: "assistant",
        content: streamText,
        created_at: new Date().toISOString(),
      });
    }
  }

  return (
    <div className="flex h-full">
      {/* Session sidebar */}
      <div className="w-64 border-r border-[var(--border)] bg-[var(--surface)] flex flex-col shrink-0">
        <div className="p-3 border-b border-[var(--border)]">
          <button
            onClick={handleNewSession}
            className="flex items-center gap-2 w-full px-3 py-2 rounded-lg text-sm bg-[var(--accent)] text-white hover:bg-[var(--accent-hover)] transition-colors cursor-pointer"
          >
            <Plus size={16} />
            New Chat
          </button>
        </div>
        <div className="flex-1 overflow-auto p-2 space-y-1">
          {sessions.map((session) => (
            <div
              key={session.id}
              className={`group flex items-center gap-2 px-3 py-2 rounded-lg text-sm cursor-pointer transition-colors ${
                session.id === activeSessionId
                  ? "bg-[var(--accent)] text-white"
                  : "text-[var(--text-secondary)] hover:bg-[var(--surface-raised)]"
              }`}
              onClick={() => {
                setActiveSessionId(session.id);
                resetStream();
              }}
            >
              <MessageSquare size={14} className="shrink-0" />
              <span className="truncate flex-1">
                {session.title ?? "New Chat"}
              </span>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  handleDeleteSession(session.id);
                }}
                className="opacity-0 group-hover:opacity-100 hover:text-[var(--danger)] transition-opacity shrink-0 cursor-pointer"
              >
                <Trash2 size={14} />
              </button>
            </div>
          ))}
        </div>
      </div>

      {/* Chat area */}
      <div className="flex flex-col flex-1 min-w-0">
        {/* Header */}
        <div className="px-6 py-3 border-b border-[var(--border)]">
          <h1 className="text-lg font-bold text-[var(--text-primary)] flex items-center gap-2">
            <MessageSquare size={18} />
            {activeSession?.title ?? "Chat"}
          </h1>
          <p className="text-xs text-[var(--text-secondary)] mt-0.5">
            Ask questions across all your meetings
          </p>
        </div>

        {/* Messages */}
        <div ref={scrollRef} className="flex-1 overflow-auto px-6 py-4 space-y-4">
          {displayMessages.length === 0 && !loading && (
            <div className="flex flex-col items-center justify-center h-full gap-4 text-[var(--text-secondary)]">
              <MessageSquare size={48} className="opacity-30" />
              <p>Ask a question about your meetings</p>
              <div className="flex flex-wrap gap-2 justify-center max-w-md">
                {[
                  "What were the key decisions this week?",
                  "Summarize my last meeting",
                  "What action items are pending?",
                ].map((suggestion) => (
                  <button
                    key={suggestion}
                    onClick={() => setInput(suggestion)}
                    className="px-3 py-1.5 bg-[var(--surface)] border border-[var(--border)] rounded-lg text-xs hover:border-[var(--accent)] transition-colors cursor-pointer"
                  >
                    {suggestion}
                  </button>
                ))}
              </div>
            </div>
          )}

          {displayMessages.map((msg, i) => (
            <div
              key={msg.id + i}
              className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}
            >
              <div
                className={`max-w-[70%] px-4 py-3 rounded-xl text-sm ${
                  msg.role === "user"
                    ? "bg-[var(--accent)] text-white"
                    : "bg-[var(--surface)] border border-[var(--border)] text-[var(--text-primary)]"
                }`}
              >
                {msg.role === "assistant" ? (
                  <ReactMarkdown
                    remarkPlugins={[remarkGfm]}
                    className="prose prose-sm prose-invert max-w-none"
                  >
                    {msg.content || (isStreaming && i === displayMessages.length - 1 ? streamText : "")}
                  </ReactMarkdown>
                ) : (
                  msg.content
                )}
              </div>
            </div>
          ))}

          {loading && (
            <p className="text-[var(--text-secondary)] text-sm">Loading...</p>
          )}
        </div>

        {/* Input */}
        <div className="px-6 py-3 border-t border-[var(--border)] bg-[var(--surface)]">
          <div className="flex items-center gap-3 max-w-3xl mx-auto">
            <input
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSend()}
              placeholder="Ask about your meetings..."
              className="flex-1 bg-[var(--background)] border border-[var(--border)] rounded-xl px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-secondary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
              disabled={isStreaming}
            />
            <button
              onClick={handleSend}
              disabled={isStreaming || !input.trim()}
              className="p-2.5 rounded-xl bg-[var(--accent)] text-white hover:bg-[var(--accent-hover)] transition-colors disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
            >
              <Send size={18} />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}