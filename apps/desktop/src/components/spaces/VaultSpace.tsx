import { useState } from "react";

/**
 * Lightweight notes vault for the Découverte tier. Pro tier adds
 * structured DP folders, encryption metadata, and version history.
 */
export function VaultSpace() {
  const [notes, setNotes] = useState<string>("");

  return (
    <section className="h-full flex flex-col p-8 max-w-3xl mx-auto">
      <header className="mb-6">
        <h1 className="text-3xl font-semibold tracking-tight">Mon coffre-fort</h1>
        <p className="text-sm text-zinc-500 mt-2">
          Vos notes restent strictement locales sur cette machine. Les
          documents sensibles (DP en cours, productions) seront chiffrés et
          versionnés à partir du tier Pro.
        </p>
      </header>

      <textarea
        value={notes}
        onChange={(e) => setNotes(e.target.value)}
        placeholder="Notez ici vos pistes, vos doutes, vos pratiques à creuser…"
        className="flex-1 min-h-[40vh] w-full rounded-xl bg-zinc-900/40 border border-zinc-800 p-4 text-sm leading-relaxed text-zinc-200 placeholder:text-zinc-600 focus:outline-none focus:border-andrea-500 resize-none font-serif"
      />

      <footer className="text-xs text-zinc-500 mt-3">
        Sauvegarde automatique toutes les 30 secondes — à brancher en
        sous-étape suivante.
      </footer>
    </section>
  );
}
