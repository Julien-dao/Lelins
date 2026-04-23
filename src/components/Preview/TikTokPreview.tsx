import { CaptionTrim, PreviewPlaceholder } from './common';

interface Props {
  caption: string;
  thumbnailUrl: string | null;
}

export function TikTokPreview({ caption, thumbnailUrl }: Props) {
  return (
    <div className="relative aspect-[9/16] w-[300px] overflow-hidden rounded-2xl border border-white/10 bg-black text-white">
      {thumbnailUrl ? (
        <img src={thumbnailUrl} alt="" className="absolute inset-0 h-full w-full object-cover" />
      ) : (
        <PreviewPlaceholder aspectClass="aspect-[9/16]" widthClass="w-full" />
      )}

      {/* Top bar */}
      <div className="absolute inset-x-0 top-0 flex items-center justify-center gap-6 bg-gradient-to-b from-black/50 to-transparent px-4 py-3 text-xs">
        <span className="text-neutral-300">Suivis</span>
        <span className="font-semibold">Pour toi</span>
      </div>

      {/* Right rail */}
      <div className="absolute bottom-20 right-2 flex flex-col items-center gap-4 text-white">
        <div className="h-10 w-10 rounded-full bg-gradient-to-tr from-cyan-400 via-pink-500 to-red-500 p-[2px]">
          <div className="h-full w-full rounded-full bg-black" />
        </div>
        <div className="text-center text-xs">
          <div>♥</div>
          <div>128K</div>
        </div>
        <div className="text-center text-xs">
          <div>💬</div>
          <div>2 340</div>
        </div>
        <div className="text-center text-xs">
          <div>🔖</div>
          <div>8K</div>
        </div>
        <div className="text-center text-xs">
          <div>↗</div>
          <div>Partager</div>
        </div>
      </div>

      {/* Bottom caption */}
      <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 via-black/40 to-transparent px-3 pb-4 pt-8 text-xs">
        <div className="font-semibold">@votre_compte</div>
        <div className="mt-1 line-clamp-3 whitespace-pre-wrap leading-snug">
          <CaptionTrim caption={caption || 'Votre légende TikTok…'} limit={150} />
        </div>
        <div className="mt-2 flex items-center gap-2 text-[10px] text-neutral-300">
          <span>🎵</span>
          <span>Son original — votre_compte</span>
        </div>
      </div>
    </div>
  );
}
