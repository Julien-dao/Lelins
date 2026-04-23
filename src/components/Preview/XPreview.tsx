import { CaptionTrim, PreviewPlaceholder } from './common';

interface Props {
  caption: string;
  thumbnailUrl: string | null;
}

export function XPreview({ caption, thumbnailUrl }: Props) {
  return (
    <div className="w-[360px] rounded-2xl border border-white/10 bg-black p-3 text-white">
      <div className="flex gap-3">
        <div className="h-10 w-10 shrink-0 rounded-full bg-neutral-700" />
        <div className="flex-1 text-sm">
          <div className="flex items-center gap-1">
            <span className="font-bold">Votre compte</span>
            <span className="text-neutral-500">@votre_compte · 1 min</span>
          </div>
          <div className="mt-1 whitespace-pre-wrap leading-snug">
            <CaptionTrim caption={caption || 'Votre post sur X…'} limit={280} />
          </div>
          <div className="mt-2 overflow-hidden rounded-xl border border-white/10">
            {thumbnailUrl ? (
              <img src={thumbnailUrl} alt="" className="aspect-[16/9] w-full object-cover" />
            ) : (
              <PreviewPlaceholder aspectClass="aspect-[16/9]" widthClass="w-full" />
            )}
          </div>
          <div className="mt-2 flex justify-between text-xs text-neutral-400">
            <span>💬 42</span>
            <span>🔁 12</span>
            <span>♡ 128</span>
            <span>📊 4,2k</span>
            <span>↗</span>
          </div>
        </div>
      </div>
    </div>
  );
}
