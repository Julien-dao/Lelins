import { useEffect, useRef, useState } from "react";
import { ipc, ChatHistoryEntry, ModelChoice } from "../lib/ipc";

/**
 * Minimal text chat used to validate the LLM round-trip against a local
 * Ollama daemon. Real onboarding + voice + RAG arrive in steps 3-6.
 */
export function ChatPanel() {
  const [history, setHistory] = useState<ChatHistoryEntry[]>([]);
  const [draft, setDraft] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [model, setModel] = useState<ModelChoice | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    ipc
      .recommendedModel()
      .then(setModel)
      .catch((e) => setError(`hardware: ${e}`));
    ipc.chatHistory().then(setHistory).catch(() => {});
  }, []);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [history, pending]);

  async function send() {
    const text = draft.trim();
    if (!text || pending) return;
    setPending(true);
    setError(null);
    setDraft("");
    setHistory((h) => [...h, { role: "user", content: text }]);
    try {
      const reply = await ipc.chatSendText(text);
      setHistory((h) => [...h, { role: "assistant", content: reply.answer }]);
    } catch (e) {
      setError(typeof e === "string" ? e : `Erreur : ${e}`);
    } finally {
      setPending(false);
    }
  }

  async function reset() {
    await ipc.chatReset();
    setHistory([]);
    setError(null);
  }

  return (
    <section className="w-full max-w-3xl mx-auto flex flex-col h-screen p-6 gap-4">
      <header className="flex items-baseline justify-between">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">ANDREA</h1>
          {model && (
            <p className="text-xs text-zinc-500 mt-1">
              Moteur : {model.display_name}
            </p>
          )}
        </div>
        <button
          type="button"
          onClick={reset}
          className="text-xs text-zinc-400 hover:text-zinc-200 transition-colors"
        >
          Réinitialiser
        </button>
      </header>

      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto rounded-xl bg-zinc-900/40 border border-zinc-800 p-4 space-y-4"
      >
        {history.length === 0 && !pending && (
          <p className="text-zinc-500 text-sm">
            Posez votre première question à ANDREA pour commencer.
          </p>
        )}

        {history.map((msg, i) => (
          <Bubble key={i} role={msg.role}>{msg.content}</Bubble>
        ))}

        {pending && (
          <Bubble role="assistant" muted>
            ANDREA réfléchit…
          </Bubble>
        )}
      </div>

      {error && (
        <div className="text-sm text-red-400 bg-red-950/40 border border-red-900 rounded-md p-2">
          {error}
        </div>
      )}

      <div className="flex gap-2">
        <input
          type="text"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              send();
            }
          }}
          placeholder="Écrivez votre message…"
          disabled={pending}
          className="flex-1 rounded-md bg-zinc-900 border border-zinc-700 px-3 py-2 text-sm focus:outline-none focus:border-andrea-500 disabled:opacity-50"
        />
        <button
          type="button"
          onClick={send}
          disabled={pending || !draft.trim()}
          className="rounded-md bg-andrea-600 hover:bg-andrea-500 disabled:bg-zinc-700 disabled:text-zinc-500 px-4 py-2 text-sm font-medium transition-colors"
        >
          Envoyer
        </button>
      </div>
    </section>
  );
}

function Bubble({
  role,
  muted,
  children,
}: {
  role: "user" | "assistant" | "system";
  muted?: boolean;
  children: React.ReactNode;
}) {
  const isUser = role === "user";
  return (
    <div className={`flex ${isUser ? "justify-end" : "justify-start"}`}>
      <div
        className={[
          "max-w-[80%] rounded-2xl px-4 py-2.5 text-sm leading-relaxed whitespace-pre-wrap",
          isUser
            ? "bg-andrea-600 text-white"
            : "bg-zinc-800/70 text-zinc-100 border border-zinc-700/60",
          muted ? "italic text-zinc-400" : "",
        ].join(" ")}
      >
        {children}
      </div>
    </div>
  );
}
