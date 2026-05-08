import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

/**
 * Step 1 placeholder UI. The real onboarding flow is built in step 5.
 * For now we just verify the Tauri ↔ React IPC works end-to-end via the
 * `ping` command, and we render the ANDREA brand stack.
 */
export default function App() {
  const [pong, setPong] = useState<string>("…");

  useEffect(() => {
    invoke<string>("ping")
      .then((value) => setPong(value))
      .catch((err) => setPong(`error: ${err}`));
  }, []);

  return (
    <main className="min-h-screen flex flex-col items-center justify-center p-12 text-center">
      <div className="space-y-8 max-w-xl">
        <div className="inline-flex items-center justify-center w-20 h-20 rounded-2xl bg-gradient-to-br from-andrea-700 to-lagon-600">
          <span className="text-4xl font-semibold text-white tracking-tight">
            A
          </span>
        </div>

        <div className="space-y-3">
          <h1 className="text-4xl font-semibold tracking-tight">ANDREA</h1>
          <p className="text-lg text-zinc-400">
            Votre formateur virtuel pour le titre FPA
          </p>
        </div>

        <div className="text-sm text-zinc-500 font-mono">
          IPC&nbsp;: {pong}
        </div>

        <p className="text-xs text-zinc-600 max-w-md mx-auto leading-relaxed">
          ANDREA est un outil d'accompagnement à la préparation du Titre
          Professionnel Formateur Professionnel d'Adultes (RNCP&nbsp;n°37275).
          ANDREA n'est pas un organisme certificateur ni un organisme de
          formation habilité.
        </p>
      </div>
    </main>
  );
}
