import { CaptionTrim, PreviewPlaceholder } from './common';
import type { SocialNetwork } from '../../types';

interface Props {
  caption: string;
  thumbnailUrl: string | null;
  variant: SocialNetwork;
}

export function InstagramPreview({ caption, thumbnailUrl, variant }: Props) {
  const isVertical = variant === 'instagram-reel' || variant === 'instagram-story';
  const aspect = variant === 'instagram-feed' ? 'aspect-[4/5]' : 'aspect-[9/16]';

  return (
    <div className="w-[320px] overflow-hidden rounded-2xl border border-white/10 bg-black text-white">
      <div className="flex items-center justify-between px-3 py-2">
        <div className="flex items-center gap-2">
          <div className="h-7 w-7 rounded-full bg-gradient-to-tr from-yellow-400 via-pink-500 to-purple-600 p-[2px]">
            <div className="h-full w-full rounded-full bg-black" />
          </div>
          <div className="text-xs">
            <div className="font-semibold">votre_compte</div>
            <div className="text-[10px] text-neutral-400">Original audio</div>
          </div>
        </div>
        <div className="text-xl leading-none">···</div>
      </div>

      <div className={`relative ${aspect} w-full bg-neutral-900`}>
        {thumbnailUrl ? (
          <img src={thumbnailUrl} alt="" className="h-full w-full object-cover" />
        ) : (
          <PreviewPlaceholder aspectClass={aspect} widthClass="w-full" />
        )}
        {isVertical && (
          <div className="absolute right-2 top-1/2 flex -translate-y-1/2 flex-col items-center gap-4 text-white drop-shadow">
            <div className="text-xl">♥</div>
            <div className="text-xl">💬</div>
            <div className="text-xl">↗</div>
            <div className="text-xl">⋯</div>
          </div>
        )}
      </div>

      <div className="px-3 py-2">
        <div className="flex items-center gap-4 text-lg">
          <span>♡</span>
          <span>💬</span>
          <span>↗</span>
          <span className="ml-auto">🔖</span>
        </div>
        <div className="mt-1 text-[11px] font-semibold">1 234 J'aime</div>
        <div className="mt-1 whitespace-pre-wrap text-[11px] leading-snug">
          <span className="mr-1 font-semibold">votre_compte</span>
          <CaptionTrim caption={caption || 'Votre légende apparaîtra ici…'} limit={125} />
        </div>
        <div className="mt-1 text-[10px] text-neutral-500">Voir les 42 commentaires</div>
      </div>
    </div>
  );
}
