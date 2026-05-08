import { ALL_CCP, ALL_CP, REFERENTIEL_VERSION, RNCP_CODE } from "../../lib/referentiel";

/**
 * "Mes cours" — Découverte tier.
 *
 * Grid 4 CCP × leur CP, with a tiny visual progression placeholder
 * (Bloom 0..4). Live progression numbers come in a follow-up wired to
 * the `progression` SQLite table.
 */
export function CoursesSpace() {
  return (
    <section className="h-full overflow-y-auto p-8 max-w-5xl mx-auto">
      <header className="mb-8">
        <h1 className="text-3xl font-semibold tracking-tight">Mes cours</h1>
        <p className="text-sm text-zinc-500 mt-2">
          Le titre {RNCP_CODE} en quatre certificats de compétences
          professionnelles. Progressez à votre rythme. Source :{" "}
          {REFERENTIEL_VERSION}.
        </p>
      </header>

      <div className="space-y-8">
        {ALL_CCP.map((ccp) => {
          const cps = ALL_CP.filter((c) => c.ccp === ccp.code);
          return (
            <article
              key={ccp.code}
              className="rounded-xl border border-zinc-800/60 bg-zinc-900/30 overflow-hidden"
            >
              <header className="px-5 py-4 bg-gradient-to-r from-andrea-900/30 to-transparent border-b border-zinc-800/60">
                <div className="flex items-baseline gap-3">
                  <span className="text-xs font-mono text-andrea-400">
                    {ccp.code}
                  </span>
                  <h2 className="text-lg font-semibold">{ccp.title}</h2>
                </div>
                <p className="text-xs text-zinc-500 mt-1.5 leading-relaxed">
                  {ccp.summary}
                </p>
              </header>
              <ul className="divide-y divide-zinc-800/60">
                {cps.map((cp) => (
                  <li
                    key={cp.code}
                    className="px-5 py-3 flex items-start gap-3 hover:bg-zinc-900/40 transition-colors"
                  >
                    <span className="font-mono text-xs text-zinc-500 mt-0.5 w-12 shrink-0">
                      {cp.code}
                    </span>
                    <span className="flex-1 text-sm text-zinc-300 leading-relaxed">
                      {cp.title}
                    </span>
                    <ProgressionDot level={0} />
                  </li>
                ))}
              </ul>
            </article>
          );
        })}
      </div>
    </section>
  );
}

function ProgressionDot({ level }: { level: number }) {
  // 0 = not started, 4 = mastered (Bloom revised). Placeholder until the
  // progression hook wires up.
  const cells = Array.from({ length: 4 }, (_, i) => i < level);
  return (
    <div
      className="flex gap-0.5 mt-1.5"
      aria-label={`Niveau Bloom ${level} sur 4`}
    >
      {cells.map((on, i) => (
        <span
          key={i}
          className={[
            "w-1.5 h-1.5 rounded-full",
            on ? "bg-lagon-500" : "bg-zinc-700",
          ].join(" ")}
        />
      ))}
    </div>
  );
}
