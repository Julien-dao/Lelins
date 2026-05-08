import { ALL_CCP } from "../../lib/referentiel";

/** "Ma progression" — overview of where the apprenant stands. */
export function ProgressionSpace() {
  return (
    <section className="h-full overflow-y-auto p-8 max-w-3xl mx-auto">
      <header className="mb-8">
        <h1 className="text-3xl font-semibold tracking-tight">Ma progression</h1>
        <p className="text-sm text-zinc-500 mt-2">
          Vue d'ensemble de votre avancée vers le titre. Les indicateurs
          se rempliront au fur et à mesure de vos sessions avec ANDREA.
        </p>
      </header>

      <div className="grid grid-cols-2 gap-4 mb-10">
        <Stat label="Compétences travaillées" value="0 / 13" />
        <Stat label="Sessions tenues" value="0" />
        <Stat label="Documents soumis" value="0" />
        <Stat label="Prêt·e pour l'épreuve" value="—" />
      </div>

      <h2 className="text-base font-semibold mb-3">Par CCP</h2>
      <ul className="space-y-2">
        {ALL_CCP.map((ccp) => (
          <li
            key={ccp.code}
            className="rounded-lg border border-zinc-800/60 bg-zinc-900/30 px-4 py-3 flex items-center gap-3"
          >
            <span className="font-mono text-xs text-andrea-400 w-12">
              {ccp.code}
            </span>
            <span className="flex-1 text-sm">{ccp.title}</span>
            <div className="w-32 h-1.5 rounded-full bg-zinc-800 overflow-hidden">
              <div className="h-full w-0 bg-gradient-to-r from-andrea-600 to-lagon-500" />
            </div>
            <span className="text-xs text-zinc-500 w-10 text-right">0 %</span>
          </li>
        ))}
      </ul>
    </section>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-xl border border-zinc-800/60 bg-zinc-900/30 px-5 py-4">
      <div className="text-xs uppercase tracking-wider text-zinc-500">
        {label}
      </div>
      <div className="text-2xl font-semibold mt-1.5 tabular-nums">{value}</div>
    </div>
  );
}
