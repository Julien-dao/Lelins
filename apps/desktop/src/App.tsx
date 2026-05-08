import { useEffect, useState } from "react";
import { ipc, ModelChoice } from "./lib/ipc";
import { SPACES, SpaceId, Tier, isUnlocked } from "./lib/spaces";
import { Layout } from "./components/Layout";
import { ChatPanel } from "./components/ChatPanel";
import { CoursesSpace } from "./components/spaces/CoursesSpace";
import { ProgressionSpace } from "./components/spaces/ProgressionSpace";
import { VaultSpace } from "./components/spaces/VaultSpace";
import { SettingsSpace } from "./components/spaces/SettingsSpace";
import { LockedSpace } from "./components/spaces/LockedSpace";

/**
 * Top-level shell. Step 5's full onboarding wizard slots in front of this
 * once the desktop app detects an empty profile; for now the shell boots
 * straight into the chat panel.
 */
export default function App() {
  const [current, setCurrent] = useState<SpaceId>("chat");
  const [model, setModel] = useState<ModelChoice | null>(null);
  const [ragCount, setRagCount] = useState<number | null>(null);

  // Hardcoded for v1 dev — replaced by the active license tier in step 7.
  const userTier: Tier = "DECO";

  useEffect(() => {
    ipc.recommendedModel().then(setModel).catch(() => {});
    const tick = () => ipc.ragStatus().then(setRagCount).catch(() => {});
    tick();
    const id = window.setInterval(tick, 1500);
    return () => window.clearInterval(id);
  }, []);

  const space = SPACES.find((s) => s.id === current) ?? SPACES[0];

  let body: React.ReactNode;
  if (!isUnlocked(space.tier, userTier)) {
    body = <LockedSpace space={space} userTier={userTier} />;
  } else {
    switch (current) {
      case "chat":
        body = <ChatPanel />;
        break;
      case "courses":
        body = <CoursesSpace />;
        break;
      case "progression":
        body = <ProgressionSpace />;
        break;
      case "vault":
        body = <VaultSpace />;
        break;
      case "settings":
        body = <SettingsSpace userTier={userTier} />;
        break;
      default:
        // Unimplemented Pro/Maître spaces fall back to the locked CTA so
        // the user always sees something coherent.
        body = <LockedSpace space={space} userTier={userTier} />;
    }
  }

  return (
    <div className="min-h-screen bg-bg-deep text-zinc-100">
      <Layout
        current={current}
        onNavigate={setCurrent}
        userTier={userTier}
        ragCount={ragCount}
        modelLabel={model?.display_name ?? null}
      >
        {body}
      </Layout>
    </div>
  );
}
