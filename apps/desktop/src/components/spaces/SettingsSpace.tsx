import { useEffect, useState } from "react";
import { ipc, HardwareProfile, ModelChoice } from "../../lib/ipc";
import { tierLabel } from "../../lib/spaces";

type Tier = "DECO" | "PRO" | "MAIT" | "BNDL";

type Props = {
  userTier: Tier;
};

export function SettingsSpace({ userTier }: Props) {
  const [hardware, setHardware] = useState<HardwareProfile | null>(null);
  const [model, setModel] = useState<ModelChoice | null>(null);
  const [licenseKey, setLicenseKey] = useState("");
  const [licenseEmail, setLicenseEmail] = useState("");
  const [licenseStatus, setLicenseStatus] = useState<string | null>(null);

  useEffect(() => {
    ipc.hardwareProfile().then(setHardware).catch(() => {});
    ipc.recommendedModel().then(setModel).catch(() => {});
  }, []);

  async function activateLicense() {
    setLicenseStatus(null);
    try {
      const summary = await ipc.licenseValidate(licenseKey, licenseEmail);
      setLicenseStatus(`Clé ${summary.tier} validée pour ${licenseEmail}.`);
    } catch (e) {
      setLicenseStatus(typeof e === "string" ? e : `Erreur : ${e}`);
    }
  }

  return (
    <section className="h-full overflow-y-auto p-8 max-w-2xl mx-auto space-y-8">
      <header>
        <h1 className="text-3xl font-semibold tracking-tight">Paramètres</h1>
        <p className="text-sm text-zinc-500 mt-2">
          Réglez votre profil, votre licence et l'apparence de l'application.
        </p>
      </header>

      <Card title="Licence">
        <div className="text-sm text-zinc-400">
          Tier actif : <span className="text-zinc-100">{tierLabel(userTier)}</span>
        </div>
        <div className="mt-4 space-y-3">
          <input
            type="text"
            value={licenseKey}
            onChange={(e) => setLicenseKey(e.target.value)}
            placeholder="ANDREA-DECO-XXXXX-XXXXX-XXXXX-XXXXX"
            className="w-full rounded-md bg-zinc-900 border border-zinc-700 px-3 py-2 text-sm font-mono"
          />
          <input
            type="email"
            value={licenseEmail}
            onChange={(e) => setLicenseEmail(e.target.value)}
            placeholder="email d'achat"
            className="w-full rounded-md bg-zinc-900 border border-zinc-700 px-3 py-2 text-sm"
          />
          <button
            type="button"
            onClick={activateLicense}
            disabled={!licenseKey.trim() || !licenseEmail.trim()}
            className="rounded-md bg-andrea-600 hover:bg-andrea-500 disabled:bg-zinc-700 disabled:text-zinc-500 px-4 py-2 text-sm font-medium transition-colors"
          >
            Activer
          </button>
          {licenseStatus && (
            <div className="text-sm text-zinc-300 bg-zinc-900/40 border border-zinc-800 rounded-md p-2">
              {licenseStatus}
            </div>
          )}
        </div>
      </Card>

      <Card title="Matériel détecté">
        {hardware ? (
          <ul className="text-sm text-zinc-300 space-y-1.5">
            <li>RAM totale : {hardware.total_ram_gb} Go</li>
            <li>Cœurs logiques : {hardware.logical_cpus}</li>
            <li>Architecture : {hardware.arch} ({hardware.os})</li>
            {hardware.cpu_brand && <li>CPU : {hardware.cpu_brand}</li>}
          </ul>
        ) : (
          <p className="text-sm text-zinc-500">Détection en cours…</p>
        )}
      </Card>

      <Card title="Modèle recommandé">
        {model ? (
          <div className="text-sm text-zinc-300 space-y-2">
            <div className="font-medium">{model.display_name}</div>
            <div className="text-zinc-500 text-xs">
              ~{model.disk_size_gb} Go disque · ~{model.runtime_ram_gb} Go en
              fonctionnement
            </div>
            <p className="text-zinc-400 leading-relaxed">{model.blurb}</p>
          </div>
        ) : (
          <p className="text-sm text-zinc-500">Calcul…</p>
        )}
      </Card>

      <Card title="Données">
        <p className="text-sm text-zinc-400 leading-relaxed">
          Vos données restent strictement locales sur cette machine.
          L'export d'une sauvegarde chiffrée et la restauration seront
          ajoutés ici à l'étape suivante.
        </p>
        <div className="mt-3 flex gap-2">
          <button
            type="button"
            disabled
            className="rounded-md border border-zinc-700 text-zinc-400 px-3 py-1.5 text-xs disabled:opacity-50"
          >
            Exporter (.andrea-backup)
          </button>
          <button
            type="button"
            disabled
            className="rounded-md border border-zinc-700 text-zinc-400 px-3 py-1.5 text-xs disabled:opacity-50"
          >
            Restaurer
          </button>
        </div>
      </Card>
    </section>
  );
}

function Card({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="rounded-xl border border-zinc-800/60 bg-zinc-900/30 p-5">
      <h2 className="text-base font-semibold mb-3">{title}</h2>
      {children}
    </div>
  );
}
