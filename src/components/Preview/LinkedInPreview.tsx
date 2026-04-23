import { CaptionTrim, PreviewPlaceholder } from './common';

interface Props {
  caption: string;
  thumbnailUrl: string | null;
}

export function LinkedInPreview({ caption, thumbnailUrl }: Props) {
  return (
    <div className="w-[360px] rounded-lg border border-white/10 bg-white text-neutral-900">
      <div className="flex items-center gap-3 p-3">
        <div className="h-10 w-10 rounded-full bg-neutral-300" />
        <div className="text-sm">
          <div className="font-semibold">Votre Nom</div>
          <div className="text-[11px] text-neutral-500">
            Créateur de contenus · 1er
          </div>
          <div className="text-[11px] text-neutral-500">1 min · 🌐</div>
        </div>
      </div>
      <div className="px-3 pb-2 text-sm leading-snug">
        <CaptionTrim caption={caption || 'Votre publication LinkedIn…'} limit={210} />
      </div>
      <div className="bg-neutral-100">
        {thumbnailUrl ? (
          <img src={thumbnailUrl} alt="" className="aspect-square w-full object-cover" />
        ) : (
          <PreviewPlaceholder aspectClass="aspect-square" widthClass="w-full" />
        )}
      </div>
      <div className="flex items-center justify-between border-t border-neutral-200 px-3 py-1 text-[11px] text-neutral-500">
        <span>👍❤️🎉 248</span>
        <span>42 commentaires</span>
      </div>
      <div className="grid grid-cols-4 border-t border-neutral-200 text-xs font-medium text-neutral-600">
        <button className="py-2 hover:bg-neutral-100">👍 J'aime</button>
        <button className="py-2 hover:bg-neutral-100">💬 Commenter</button>
        <button className="py-2 hover:bg-neutral-100">🔁 Republier</button>
        <button className="py-2 hover:bg-neutral-100">↗ Envoyer</button>
      </div>
    </div>
  );
}
