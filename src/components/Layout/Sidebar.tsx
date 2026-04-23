import { useState } from 'react';

interface Tab {
  id: string;
  label: string;
  icon: string;
  content: React.ReactNode;
}

interface Props {
  tabs: Tab[];
  side: 'left' | 'right';
  initialTab?: string;
}

export function Sidebar({ tabs, side, initialTab }: Props) {
  const [active, setActive] = useState(initialTab ?? tabs[0].id);
  const activeTab = tabs.find((t) => t.id === active) ?? tabs[0];
  return (
    <aside
      className={`flex h-full w-[320px] shrink-0 flex-col bg-neutral-950 ${
        side === 'left' ? 'border-r' : 'border-l'
      } border-white/10`}
    >
      <div className="flex gap-1 border-b border-white/10 p-2">
        {tabs.map((t) => {
          const isActive = t.id === active;
          return (
            <button
              key={t.id}
              onClick={() => setActive(t.id)}
              className={`flex-1 rounded-md px-2 py-1.5 text-xs font-medium transition ${
                isActive
                  ? 'bg-brand-500/20 text-brand-300 ring-1 ring-brand-500/50'
                  : 'text-neutral-400 hover:bg-white/5 hover:text-neutral-200'
              }`}
            >
              <span className="mr-1">{t.icon}</span>
              {t.label}
            </button>
          );
        })}
      </div>
      <div className="flex-1 overflow-y-auto p-3">{activeTab.content}</div>
    </aside>
  );
}
