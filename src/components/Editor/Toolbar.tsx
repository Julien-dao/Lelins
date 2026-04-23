import { useRef } from 'react';
import { useEditor } from '../../store/editorStore';
import { FORMAT_LIST } from '../../lib/formats';
import type { MediaSource, SocialNetwork } from '../../types';

interface Props {
  onExport: (format: 'png' | 'jpeg' | 'mp4') => void;
  onShare: () => void;
  onPublish: () => void;
  publishing: boolean;
  exporting: boolean;
  exportProgress: number | null;
}

export function Toolbar({
  onExport,
  onShare,
  onPublish,
  publishing,
  exporting,
  exportProgress,
}: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const media = useEditor((s) => s.media);
  const setMedia = useEditor((s) => s.setMedia);
  const format = useEditor((s) => s.format);
  const setFormat = useEditor((s) => s.setFormat);
  const selectedAccountIds = useEditor((s) => s.selectedAccountIds);

  const onFile = (file: File) => {
    const isVideo = file.type.startsWith('video/');
    const url = URL.createObjectURL(file);
    if (isVideo) {
      const v = document.createElement('video');
      v.preload = 'metadata';
      v.onloadedmetadata = () => {
        const media: MediaSource = {
          kind: 'video',
          src: url,
          file,
          naturalWidth: v.videoWidth,
          naturalHeight: v.videoHeight,
          duration: v.duration,
          name: file.name,
        };
        setMedia(media);
      };
      v.src = url;
    } else {
      const img = new Image();
      img.onload = () => {
        setMedia({
          kind: 'image',
          src: url,
          naturalWidth: img.naturalWidth,
          naturalHeight: img.naturalHeight,
          name: file.name,
        });
      };
      img.src = url;
    }
  };

  const isVideo = media?.kind === 'video';

  return (
    <div className="flex flex-wrap items-center gap-2 border-b border-white/10 bg-neutral-950/80 px-4 py-2 backdrop-blur">
      <div className="flex items-center gap-2">
        <div className="text-sm font-semibold tracking-tight">
          <span className="text-brand-400">Lelins</span>{' '}
          <span className="text-neutral-400">Studio</span>
        </div>
      </div>

      <div className="mx-3 h-5 w-px bg-white/10" />

      <button className="btn-outline" onClick={() => inputRef.current?.click()}>
        📁 Importer
      </button>
      <input
        ref={inputRef}
        type="file"
        accept="image/*,video/*"
        className="hidden"
        onChange={(e) => {
          const f = e.target.files?.[0];
          if (f) onFile(f);
          e.currentTarget.value = '';
        }}
      />

      <select
        className="input w-auto"
        value={format}
        onChange={(e) => setFormat(e.target.value as SocialNetwork)}
      >
        {FORMAT_LIST.map((f) => (
          <option key={f.id} value={f.id}>
            {f.label} · {f.width}×{f.height}
          </option>
        ))}
      </select>

      {exporting && (
        <div className="flex items-center gap-2 rounded-lg bg-brand-500/10 px-3 py-1 text-xs text-brand-300 ring-1 ring-brand-500/30">
          <span>Export…</span>
          {exportProgress !== null && (
            <div className="h-1 w-24 overflow-hidden rounded-full bg-white/10">
              <div
                className="h-full bg-brand-500 transition-all"
                style={{ width: `${exportProgress}%` }}
              />
            </div>
          )}
        </div>
      )}

      <div className="ml-auto flex flex-wrap items-center gap-2">
        {isVideo ? (
          <button
            className="btn-outline"
            onClick={() => onExport('mp4')}
            disabled={!media || exporting}
            title="Exporter en MP4"
          >
            ⤓ MP4
          </button>
        ) : (
          <>
            <button
              className="btn-outline"
              onClick={() => onExport('png')}
              disabled={!media || exporting}
              title="Télécharger en PNG"
            >
              ⤓ PNG
            </button>
            <button
              className="btn-outline"
              onClick={() => onExport('jpeg')}
              disabled={!media || exporting}
              title="Télécharger en JPEG"
            >
              ⤓ JPG
            </button>
          </>
        )}
        <button className="btn-outline" onClick={onShare} disabled={!media || exporting}>
          ↗ Partager
        </button>
        <button
          className="btn-primary"
          onClick={onPublish}
          disabled={!media || selectedAccountIds.length === 0 || publishing || exporting}
        >
          {publishing ? 'Publication…' : `Publier (${selectedAccountIds.length})`}
        </button>
      </div>
    </div>
  );
}
