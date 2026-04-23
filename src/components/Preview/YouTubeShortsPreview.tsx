import { CaptionTrim, PreviewPlaceholder } from './common';

interface Props {
  caption: string;
  thumbnailUrl: string | null;
}

export function YouTubeShortsPreview({ caption, thumbnailUrl }: Props) {
  return (
    <div className="relative aspect-[9/16] w-[300px] overflow-hidden rounded-2xl border border-white/10 bg-black text-white">
      {thumbnailUrl ? (
        <img src={thumbnailUrl} alt="" className="absolute inset-0 h-full w-full object-cover" />
      ) : (
        <PreviewPlaceholder aspectClass="aspect-[9/16]" widthClass="w-full" />
      )}

      <div className="absolute left-0 right-0 top-0 flex items-center justify-between px-3 py-3 text-xs">
        <div className="flex items-center gap-1 font-bold">
          <span className="text-red-500">▶</span>
          <span>Shorts</span>
        </div>
        <div className="flex gap-3 text-lg">
          <span>🔎</span>
          <span>⋮</span>
        </div>
      </div>

      <div className="absolute bottom-16 right-2 flex flex-col items-center gap-4">
        <div className="text-center text-xs">
          <div className="text-xl">👍</div>
          <div>24K</div>
        </div>
        <div className="text-center text-xs">
          <div className="text-xl">👎</div>
          <div>Je n'aime pas</div>
        </div>
        <div className="text-center text-xs">
          <div className="text-xl">💬</div>
          <div>1 240</div>
        </div>
        <div className="text-center text-xs">
          <div className="text-xl">↗</div>
          <div>Partager</div>
        </div>
      </div>

      <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 to-transparent px-3 pb-4 pt-8 text-xs">
        <div className="flex items-center gap-2">
          <div className="h-6 w-6 rounded-full bg-red-500" />
          <div className="font-semibold">@votre_compte</div>
          <button className="ml-auto rounded-full bg-white px-3 py-0.5 text-[11px] font-semibold text-black">
            S'abonner
          </button>
        </div>
        <div className="mt-2 line-clamp-2 whitespace-pre-wrap leading-snug">
          <CaptionTrim caption={caption || 'Votre titre Shorts…'} limit={100} />
        </div>
      </div>
    </div>
  );
}
