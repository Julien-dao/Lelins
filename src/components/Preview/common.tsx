import { useMemo } from 'react';

export function highlightCaption(text: string) {
  // Split while preserving matched tokens (#tag / @mention / URL-ish).
  const regex = /(#[\p{L}\p{N}_]+|@[\p{L}\p{N}_.]+|https?:\/\/\S+)/gu;
  const parts = text.split(regex);
  return parts.map((p, i) => {
    if (!p) return null;
    if (p.startsWith('#')) {
      return (
        <span key={i} className="text-sky-400">
          {p}
        </span>
      );
    }
    if (p.startsWith('@')) {
      return (
        <span key={i} className="text-sky-400">
          {p}
        </span>
      );
    }
    if (p.startsWith('http')) {
      return (
        <span key={i} className="text-sky-400 underline">
          {p}
        </span>
      );
    }
    return <span key={i}>{p}</span>;
  });
}

export function CaptionTrim({ caption, limit }: { caption: string; limit: number }) {
  const trimmed = useMemo(() => {
    if (caption.length <= limit) return { text: caption, truncated: false };
    return { text: caption.slice(0, limit) + '…', truncated: true };
  }, [caption, limit]);
  return <>{highlightCaption(trimmed.text)}</>;
}

export function PreviewPlaceholder({
  aspectClass,
  widthClass,
}: {
  aspectClass: string;
  widthClass: string;
}) {
  return (
    <div
      className={`${widthClass} ${aspectClass} flex items-center justify-center rounded-md bg-gradient-to-br from-neutral-700 to-neutral-900 text-[10px] text-neutral-500`}
    >
      Aperçu (importez une image)
    </div>
  );
}
