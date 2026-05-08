import { ChatPanel } from "./components/ChatPanel";

/**
 * Step 2 placeholder UI. The real onboarding flow is built in step 5.
 * For now we render a minimal text chat that exercises the full
 * Tauri ↔ Ollama pipeline.
 */
export default function App() {
  return (
    <main className="min-h-screen bg-bg-deep text-zinc-100">
      <ChatPanel />
    </main>
  );
}
