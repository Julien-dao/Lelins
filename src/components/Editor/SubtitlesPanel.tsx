import { useState } from 'react';
import { useEditor } from '../../store/editorStore';
import { extractAudio } from '../../lib/audio';
import { uid } from '../../lib/id';

type WhisperModel =
  | 'Xenova/whisper-tiny'
  | 'Xenova/whisper-base'
  | 'Xenova/whisper-small';

const LANGUAGES = [
  { code: '', label: 'Auto' },
  { code: 'fr', label: 'Français' },
  { code: 'en', label: 'Anglais' },
  { code: 'es', label: 'Espagnol' },
  { code: 'de', label: 'Allemand' },
  { code: 'it', label: 'Italien' },
  { code: 'pt', label: 'Portugais' },
];

const MODELS: { id: WhisperModel; label: string; size: string }[] = [
  { id: 'Xenova/whisper-tiny', label: 'Tiny (rapide)', size: '~75 Mo' },
  { id: 'Xenova/whisper-base', label: 'Base', size: '~150 Mo' },
  { id: 'Xenova/whisper-small', label: 'Small (précis)', size: '~480 Mo' },
];

function fmt(t: number): string {
  if (!Number.isFinite(t)) return '0:00.0';
  const m = Math.floor(t / 60);
  const s = (t % 60).toFixed(1);
  return `${m}:${s.padStart(4, '0')}`;
}

export function SubtitlesPanel() {
  const media = useEditor((s) => s.media);
  const segments = useEditor((s) => s.subtitleTrack);
  const setSubtitleTrack = useEditor((s) => s.setSubtitleTrack);
  const updateSegment = useEditor((s) => s.updateSubtitleSegment);
  const removeSegment = useEditor((s) => s.removeSubtitleSegment);
  const clear = useEditor((s) => s.clearSubtitleTrack);
  const style = useEditor((s) => s.subtitleStyle);
  const setStyle = useEditor((s) => s.setSubtitleStyle);
  const setVideoTime = useEditor((s) => s.setVideoTime);

  const [language, setLanguage] = useState('fr');
  const [model, setModel] = useState<WhisperModel>('Xenova/whisper-tiny');
  const [running, setRunning] = useState(false);
  const [progress, setProgress] = useState<{ status: string; detail?: string; pct?: number } | null>(
    null,
  );
  const [error, setError] = useState<string | null>(null);

  const isVideo = media?.kind === 'video';

  const generate = async () => {
    if (!isVideo) return;
    setError(null);
    setRunning(true);
    setProgress({ status: 'Préparation…' });
    try {
      setProgress({ status: 'Extraction audio…' });
      const audio = await extractAudio(media.file);
      setProgress({ status: 'Chargement du moteur Whisper…' });
      const { transcribe } = await import('../../lib/whisper');
      setProgress({ status: 'Transcription…' });
      const result = await transcribe(audio, {
        language: language || undefined,
        model,
        onProgress: (p) =>
          setProgress({
            status:
              p.status === 'loading'
                ? 'Chargement modèle…'
                : p.status === 'transcribing'
                  ? 'Transcription…'
                  : 'Terminé',
            detail: p.detail,
            pct: p.percent,
          }),
      });
      setSubtitleTrack(
        result.map((s) => ({ id: uid('sub'), start: s.start, end: s.end, text: s.text })),
      );
      setProgress({ status: 'Terminé' });
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      setProgress(null);
    } finally {
      setRunning(false);
    }
  };

  return (
    <div className="space-y-3">
      {!isVideo && (
        <p className="rounded-lg border border-dashed border-white/10 px-3 py-6 text-center text-xs text-neutral-500">
          Importez une vidéo pour générer des sous-titres automatiquement.
        </p>
      )}

      {isVideo && (
        <>
          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="label">Langue</label>
              <select
                className="input"
                value={language}
                onChange={(e) => setLanguage(e.target.value)}
                disabled={running}
              >
                {LANGUAGES.map((l) => (
                  <option key={l.code} value={l.code}>
                    {l.label}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="label">Modèle</label>
              <select
                className="input"
                value={model}
                onChange={(e) => setModel(e.target.value as WhisperModel)}
                disabled={running}
              >
                {MODELS.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.label} · {m.size}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <button className="btn-primary w-full" onClick={generate} disabled={running}>
            {running ? 'Génération…' : segments.length > 0 ? 'Régénérer' : 'Générer les sous-titres'}
          </button>

          {progress && (
            <div className="rounded-lg border border-white/10 bg-white/5 p-2 text-xs">
              <div className="flex items-center justify-between">
                <span className="font-medium">{progress.status}</span>
                {progress.pct !== undefined && (
                  <span className="tabular-nums text-neutral-400">{Math.round(progress.pct)}%</span>
                )}
              </div>
              {progress.detail && <div className="mt-0.5 text-neutral-500">{progress.detail}</div>}
            </div>
          )}

          {error && (
            <div className="rounded-lg border border-red-500/30 bg-red-500/10 p-2 text-xs text-red-300">
              {error}
            </div>
          )}

          <details className="rounded-lg border border-white/10 bg-white/5 p-2 text-xs">
            <summary className="cursor-pointer font-medium text-neutral-300">
              Style des sous-titres
            </summary>
            <div className="mt-2 space-y-2">
              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="label">Taille · {style.fontSize}</label>
                  <input
                    type="range"
                    min={28}
                    max={120}
                    step={1}
                    value={style.fontSize}
                    onChange={(e) => setStyle({ fontSize: Number(e.target.value) })}
                    className="w-full"
                  />
                </div>
                <div>
                  <label className="label">Position</label>
                  <select
                    className="input"
                    value={style.position}
                    onChange={(e) =>
                      setStyle({ position: e.target.value as 'top' | 'middle' | 'bottom' })
                    }
                  >
                    <option value="top">Haut</option>
                    <option value="middle">Milieu</option>
                    <option value="bottom">Bas</option>
                  </select>
                </div>
              </div>
              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="label">Couleur texte</label>
                  <input
                    type="color"
                    className="input h-9 cursor-pointer p-0.5"
                    value={style.color}
                    onChange={(e) => setStyle({ color: e.target.value })}
                  />
                </div>
                <div>
                  <label className="label">Couleur fond</label>
                  <input
                    type="color"
                    className="input h-9 cursor-pointer p-0.5"
                    value={style.backgroundColor}
                    onChange={(e) => setStyle({ backgroundColor: e.target.value })}
                  />
                </div>
              </div>
              <div>
                <label className="label">Fond</label>
                <div className="flex gap-2">
                  {(['none', 'box', 'pill'] as const).map((v) => (
                    <button
                      key={v}
                      className={`btn-outline flex-1 text-xs ${style.background === v ? 'ring-1 ring-brand-500' : ''}`}
                      onClick={() => setStyle({ background: v })}
                    >
                      {v === 'none' ? 'Aucun' : v === 'box' ? 'Bloc' : 'Pilule'}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          </details>

          {segments.length > 0 && (
            <div className="space-y-1">
              <div className="flex items-center justify-between">
                <span className="label mb-0">{segments.length} segments</span>
                <button
                  className="text-xs text-red-300 hover:text-red-200"
                  onClick={clear}
                >
                  Tout effacer
                </button>
              </div>
              <div className="max-h-[280px] space-y-1 overflow-y-auto">
                {segments.map((seg) => (
                  <div
                    key={seg.id}
                    className="rounded-lg border border-white/10 bg-white/5 p-2 text-xs"
                  >
                    <div className="mb-1 flex items-center gap-2">
                      <button
                        className="rounded bg-white/10 px-1.5 py-0.5 font-mono text-[10px] hover:bg-white/20"
                        onClick={() => setVideoTime(seg.start)}
                        title="Aller à ce segment"
                      >
                        {fmt(seg.start)} → {fmt(seg.end)}
                      </button>
                      <button
                        className="ml-auto rounded text-red-300 hover:bg-red-500/20"
                        onClick={() => removeSegment(seg.id)}
                        title="Supprimer"
                      >
                        ✕
                      </button>
                    </div>
                    <textarea
                      className="input min-h-[40px] text-xs"
                      value={seg.text}
                      onChange={(e) => updateSegment(seg.id, { text: e.target.value })}
                    />
                  </div>
                ))}
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
}
