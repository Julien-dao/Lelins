import { useMemo, useState } from 'react';
import { useEditor } from '../../store/editorStore';
import { FORMATS, FORMAT_LIST } from '../../lib/formats';
import type { SocialNetwork } from '../../types';
import { InstagramPreview } from './InstagramPreview';
import { TikTokPreview } from './TikTokPreview';
import { YouTubeShortsPreview } from './YouTubeShortsPreview';
import { XPreview } from './XPreview';
import { LinkedInPreview } from './LinkedInPreview';

interface Props {
  thumbnailUrl: string | null;
}

const TABS: SocialNetwork[] = [
  'instagram-feed',
  'instagram-reel',
  'tiktok',
  'youtube-shorts',
  'x',
  'linkedin',
];

export function SocialPreviews({ thumbnailUrl }: Props) {
  const caption = useEditor((s) => s.caption);
  const format = useEditor((s) => s.format);
  const [active, setActive] = useState<SocialNetwork>(format);

  // Auto-switch preview tab when user changes the target canvas format.
  useMemo(() => {
    if (FORMAT_LIST.some((f) => f.id === format)) setActive(format);
  }, [format]);

  const renderPreview = () => {
    const props = { caption, thumbnailUrl };
    switch (active) {
      case 'instagram-feed':
      case 'instagram-story':
      case 'instagram-reel':
        return <InstagramPreview variant={active} {...props} />;
      case 'tiktok':
        return <TikTokPreview {...props} />;
      case 'youtube-shorts':
        return <YouTubeShortsPreview {...props} />;
      case 'x':
        return <XPreview {...props} />;
      case 'linkedin':
        return <LinkedInPreview {...props} />;
      default:
        return null;
    }
  };

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap gap-1">
        {TABS.map((t) => {
          const spec = FORMATS[t];
          const isActive = active === t;
          return (
            <button
              key={t}
              onClick={() => setActive(t)}
              className={`rounded-md px-2.5 py-1 text-xs font-medium transition ${
                isActive
                  ? 'bg-brand-500/20 text-brand-300 ring-1 ring-brand-500/50'
                  : 'text-neutral-400 hover:bg-white/5 hover:text-neutral-200'
              }`}
            >
              {spec.short}
            </button>
          );
        })}
      </div>
      <div className="flex justify-center">{renderPreview()}</div>
    </div>
  );
}
