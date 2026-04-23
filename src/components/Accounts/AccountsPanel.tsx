import { useState } from 'react';
import { useEditor } from '../../store/editorStore';
import { FORMATS } from '../../lib/formats';
import { NETWORK_DOCS } from '../../lib/publishers';
import type { SocialNetwork } from '../../types';

const ADDABLE: SocialNetwork[] = [
  'instagram-feed',
  'instagram-reel',
  'tiktok',
  'youtube-shorts',
  'x',
  'linkedin',
  'facebook',
];

const NEEDS_EXTERNAL_ID: SocialNetwork[] = [
  'instagram-feed',
  'instagram-reel',
  'instagram-story',
  'facebook',
];

const NEEDS_MEDIA_HOST: SocialNetwork[] = [
  'instagram-feed',
  'instagram-reel',
  'instagram-story',
];

export function AccountsPanel() {
  const accounts = useEditor((s) => s.accounts);
  const selected = useEditor((s) => s.selectedAccountIds);
  const addAccount = useEditor((s) => s.addAccount);
  const removeAccount = useEditor((s) => s.removeAccount);
  const toggleAccount = useEditor((s) => s.toggleAccount);

  const [network, setNetwork] = useState<SocialNetwork>('youtube-shorts');
  const [handle, setHandle] = useState('');
  const [token, setToken] = useState('');
  const [externalId, setExternalId] = useState('');
  const [mediaHost, setMediaHost] = useState('');
  const [showToken, setShowToken] = useState(false);

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!handle.trim() || !token.trim()) return;
    addAccount({
      network,
      handle: handle.trim().replace(/^@/, ''),
      token: token.trim(),
      externalId: externalId.trim() || undefined,
      mediaHost: mediaHost.trim() || undefined,
    });
    setHandle('');
    setToken('');
    setExternalId('');
    setMediaHost('');
  };

  const needsExtId = NEEDS_EXTERNAL_ID.includes(network);
  const needsHost = NEEDS_MEDIA_HOST.includes(network);

  return (
    <div className="space-y-4">
      <div className="rounded-lg border border-amber-500/20 bg-amber-500/10 p-3 text-xs text-amber-200">
        <p className="font-semibold">Publication avec vos propres API</p>
        <p className="mt-1 text-amber-100/80">
          L'app reste 100 % locale. Pour publier en direct, collez un token OAuth que <em>vous</em>{' '}
          avez généré via votre compte développeur. Les tokens sont stockés uniquement dans
          <code className="mx-1 rounded bg-black/30 px-1">localStorage</code> de ce navigateur.
        </p>
      </div>

      <form onSubmit={submit} className="space-y-2 rounded-lg border border-white/10 bg-white/5 p-3">
        <div>
          <label className="label">Réseau</label>
          <select
            className="input"
            value={network}
            onChange={(e) => setNetwork(e.target.value as SocialNetwork)}
          >
            {ADDABLE.map((n) => (
              <option key={n} value={n}>
                {FORMATS[n].short}
              </option>
            ))}
          </select>
          <p className="mt-1 text-[10px] text-neutral-500">{NETWORK_DOCS[network].doc}</p>
        </div>
        <div>
          <label className="label">Pseudo</label>
          <input
            className="input"
            placeholder="votre_compte"
            value={handle}
            onChange={(e) => setHandle(e.target.value)}
          />
        </div>
        <div>
          <label className="label">Token OAuth</label>
          <div className="flex gap-2">
            <input
              className="input font-mono text-xs"
              type={showToken ? 'text' : 'password'}
              placeholder="Collez votre access token"
              value={token}
              onChange={(e) => setToken(e.target.value)}
              autoComplete="off"
            />
            <button
              type="button"
              className="btn-outline text-xs"
              onClick={() => setShowToken((v) => !v)}
            >
              {showToken ? '🙈' : '👁'}
            </button>
          </div>
        </div>
        {needsExtId && (
          <div>
            <label className="label">
              {network === 'facebook' ? 'Page ID' : 'IG Business Account ID'}
            </label>
            <input
              className="input font-mono text-xs"
              placeholder="1234567890"
              value={externalId}
              onChange={(e) => setExternalId(e.target.value)}
            />
          </div>
        )}
        {needsHost && (
          <div>
            <label className="label">URL publique du média (Instagram)</label>
            <input
              className="input text-xs"
              placeholder="https://votre-cdn.example/media.mp4"
              value={mediaHost}
              onChange={(e) => setMediaHost(e.target.value)}
            />
            <p className="mt-1 text-[10px] text-neutral-500">
              IG Graph API exige que le fichier soit servi depuis une URL publique. Uploadez-le
              vous-même (S3, R2, serveur local exposé, etc.) avant publication.
            </p>
          </div>
        )}
        <button type="submit" className="btn-primary w-full" disabled={!handle || !token}>
          Ajouter le compte
        </button>
      </form>

      <div className="space-y-2">
        {accounts.length === 0 && (
          <p className="rounded-lg border border-dashed border-white/10 px-3 py-4 text-center text-xs text-neutral-500">
            Aucun compte ajouté.
          </p>
        )}
        {accounts.map((a) => {
          const isSelected = selected.includes(a.id);
          return (
            <div
              key={a.id}
              className={`flex items-center gap-2 rounded-lg border px-3 py-2 text-sm ${
                isSelected
                  ? 'border-brand-500/60 bg-brand-500/10'
                  : 'border-white/10 bg-white/5'
              }`}
            >
              <input
                type="checkbox"
                checked={isSelected}
                onChange={() => toggleAccount(a.id)}
                className="accent-brand-500"
              />
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="rounded bg-white/10 px-1.5 py-0.5 text-[10px] uppercase">
                    {FORMATS[a.network].short}
                  </span>
                  <span className="truncate font-medium">@{a.handle}</span>
                </div>
                <div className="truncate text-[10px] text-neutral-500">
                  Token : •••{a.token.slice(-4)}
                  {a.externalId && ` · id ${a.externalId}`}
                </div>
              </div>
              <button
                onClick={() => removeAccount(a.id)}
                className="rounded px-1 text-xs text-red-300 hover:bg-red-500/20"
                title="Supprimer"
              >
                ✕
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
}
