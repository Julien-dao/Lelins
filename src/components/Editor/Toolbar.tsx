import { useRef } from 'react';
import { useEditor } from '../../store/editorStore';
import { FORMAT_LIST } from '../../lib/formats';
import type { SocialNetwork } from '../../types';

interface Props {
  onExport: (format: 'png' | 'jpeg') => void;
  onShare: () => void;
  onPublish: () => void;
  publishing: boolean;
}

export function Toolbar({ onExport, onShare, onPublish, publishing }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const image = useEditor((s) => s.image);
  const setImage = useEditor((s) => s.setImage);
  const format = useEditor((s) => s.format);
  const setFormat = useEditor((s) => s.setFormat);
  const selectedAccountIds = useEditor((s) => s.selectedAccountIds);

  const onFile = (file: File) => {
    const url = URL.createObjectURL(file);
    const img = new Image();
    img.onload = () => {
      setImage({
        src: url,
        naturalWidth: img.naturalWidth,
        naturalHeight: img.naturalHeight,
        name: file.name,
      });
    };
    img.src = url;
  };

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
        accept="image/*"
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

      <div className="ml-auto flex flex-wrap items-center gap-2">
        <button
          className="btn-outline"
          onClick={() => onExport('png')}
          disabled={!image}
          title="Télécharger en PNG"
        >
          ⤓ PNG
        </button>
        <button
          className="btn-outline"
          onClick={() => onExport('jpeg')}
          disabled={!image}
          title="Télécharger en JPEG"
        >
          ⤓ JPG
        </button>
        <button className="btn-outline" onClick={onShare} disabled={!image}>
          ↗ Partager
        </button>
        <button
          className="btn-primary"
          onClick={onPublish}
          disabled={!image || selectedAccountIds.length === 0 || publishing}
        >
          {publishing ? 'Publication…' : `Publier (${selectedAccountIds.length})`}
        </button>
      </div>
    </div>
  );
}
