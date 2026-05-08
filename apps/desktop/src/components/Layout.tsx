import { useEffect, useState } from "react";
import {
  MessageCircle, BookOpen, LineChart, Archive, Settings,
  FileText, Presentation, Users, UserCheck, ScrollText, Sparkles,
  Lock, Sun, Moon,
} from "lucide-react";
import { SPACES, SpaceId, Tier, isUnlocked, tierLabel } from "../lib/spaces";

const ICONS: Record<string, React.ComponentType<{ size?: number; className?: string }>> = {
  MessageCircle, BookOpen, LineChart, Archive, Settings,
  FileText, Presentation, Users, UserCheck, ScrollText, Sparkles,
};

type Theme = "light" | "dark";

type LayoutProps = {
  current: SpaceId;
  onNavigate: (id: SpaceId) => void;
  userTier: Tier;
  ragCount: number | null;
  modelLabel: string | null;
  children: React.ReactNode;
};

/**
 * Sidebar (240 px) + content. Locked spaces are visible with a 🔒 badge so
 * the user understands what's available at higher tiers.
 */
export function Layout({
  current,
  onNavigate,
  userTier,
  ragCount,
  modelLabel,
  children,
}: LayoutProps) {
  const [theme, setTheme] = useState<Theme>(() =>
    window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light",
  );

  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark");
  }, [theme]);

  return (
    <div className="flex min-h-screen">
      <aside className="w-60 shrink-0 border-r border-zinc-800/60 bg-zinc-950/40 flex flex-col">
        <header className="px-5 py-6">
          <div className="flex items-center gap-2.5">
            <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-andrea-700 to-lagon-600 flex items-center justify-center text-white font-semibold">
              A
            </div>
            <div>
              <div className="text-base font-semibold tracking-tight leading-tight">
                ANDREA
              </div>
              <div className="text-[10px] uppercase text-zinc-500 tracking-wider">
                {tierLabel(userTier)}
              </div>
            </div>
          </div>
        </header>

        <nav className="flex-1 px-2 space-y-0.5 overflow-y-auto">
          {SPACES.map((s) => {
            const unlocked = isUnlocked(s.tier, userTier);
            const Icon = ICONS[s.iconName] ?? MessageCircle;
            const active = current === s.id;
            return (
              <button
                key={s.id}
                type="button"
                onClick={() => onNavigate(s.id)}
                className={[
                  "w-full flex items-center gap-2 px-3 py-2 rounded-md text-sm transition-colors text-left",
                  active
                    ? "bg-andrea-600/20 text-andrea-200"
                    : "text-zinc-400 hover:bg-zinc-800/40 hover:text-zinc-100",
                  !unlocked ? "opacity-60" : "",
                ].join(" ")}
                aria-current={active ? "page" : undefined}
              >
                <Icon size={16} className="shrink-0" />
                <span className="flex-1 truncate">{s.label}</span>
                {!unlocked && <Lock size={12} className="text-zinc-500" />}
              </button>
            );
          })}
        </nav>

        <footer className="px-4 py-4 border-t border-zinc-800/60 space-y-2 text-[11px] text-zinc-500">
          {modelLabel && <div className="truncate">Moteur : {modelLabel}</div>}
          {ragCount !== null && (
            <div>Référentiel : {ragCount} extraits</div>
          )}
          <button
            type="button"
            onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
            className="flex items-center gap-1.5 text-zinc-400 hover:text-zinc-200 transition-colors"
          >
            {theme === "dark" ? <Sun size={12} /> : <Moon size={12} />}
            Mode {theme === "dark" ? "clair" : "sombre"}
          </button>
        </footer>
      </aside>

      <main className="flex-1 overflow-hidden">{children}</main>
    </div>
  );
}
