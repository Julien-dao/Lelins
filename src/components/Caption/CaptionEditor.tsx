import { useMemo, useRef, useState } from 'react';
import { useEditor } from '../../store/editorStore';
import { FORMATS } from '../../lib/formats';
import { SUGGESTED_HASHTAGS, getActiveToken, parseCaption, replaceToken } from '../../lib/caption';
import type { SocialNetwork } from '../../types';

const NETWORKS_FOR_COUNTER: SocialNetwork[] = [
  'instagram-feed',
  'tiktok',
  'youtube-shorts',
  'x',
  'linkedin',
];

export function CaptionEditor() {
  const caption = useEditor((s) => s.caption);
  const setCaption = useEditor((s) => s.setCaption);
  const format = useEditor((s) => s.format);

  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [caret, setCaret] = useState(0);

  const parsed = useMemo(() => parseCaption(caption), [caption]);
  const activeToken = useMemo(() => getActiveToken(caption, caret), [caption, caret]);

  const suggestions = useMemo(() => {
    if (!activeToken) return [];
    const q = activeToken.value.toLowerCase();
    if (activeToken.kind === '#') {
      return SUGGESTED_HASHTAGS.filter((h) => h.toLowerCase().startsWith(q)).slice(0, 6);
    }
    // For mentions, suggest already-used handles + a few generic placeholders.
    const seen = Array.from(new Set(parsed.mentions.filter((m) => m.toLowerCase().startsWith(q))));
    return seen.slice(0, 6);
  }, [activeToken, parsed.mentions]);

  const pickSuggestion = (value: string) => {
    if (!activeToken) return;
    const { text, caret: nextCaret } = replaceToken(caption, activeToken, value);
    setCaption(text);
    requestAnimationFrame(() => {
      const el = textareaRef.current;
      if (el) {
        el.focus();
        el.setSelectionRange(nextCaret, nextCaret);
        setCaret(nextCaret);
      }
    });
  };

  const insertAtCaret = (snippet: string) => {
    const el = textareaRef.current;
    const pos = el ? el.selectionStart : caption.length;
    const before = caption.slice(0, pos);
    const after = caption.slice(pos);
    const needsSpace = before.length > 0 && !/\s$/.test(before);
    const inserted = (needsSpace ? ' ' : '') + snippet;
    const next = before + inserted + after;
    setCaption(next);
    requestAnimationFrame(() => {
      if (el) {
        const p = before.length + inserted.length;
        el.focus();
        el.setSelectionRange(p, p);
        setCaret(p);
      }
    });
  };

  const activeFormatSpec = FORMATS[format];

  return (
    <div className="space-y-3">
      <div className="relative">
        <textarea
          ref={textareaRef}
          className="input min-h-[140px] font-medium"
          placeholder="Écrivez votre légende… Utilisez # et @ pour les hashtags et mentions."
          value={caption}
          onChange={(e) => {
            setCaption(e.target.value);
            setCaret(e.target.selectionStart);
          }}
          onKeyUp={(e) => setCaret(e.currentTarget.selectionStart)}
          onClick={(e) => setCaret(e.currentTarget.selectionStart)}
        />
        {activeToken && suggestions.length > 0 && (
          <div className="absolute left-2 right-2 top-full z-10 mt-1 overflow-hidden rounded-lg border border-white/10 bg-neutral-900 shadow-2xl">
            {suggestions.map((s) => (
              <button
                key={s}
                onMouseDown={(e) => {
                  e.preventDefault();
                  pickSuggestion(s);
                }}
                className="flex w-full items-center justify-between px-3 py-1.5 text-left text-sm hover:bg-white/5"
              >
                <span>
                  <span className="text-brand-400">{activeToken.kind}</span>
                  {s}
                </span>
              </button>
            ))}
          </div>
        )}
      </div>

      <div className="flex flex-wrap gap-2">
        <button className="btn-outline text-xs" onClick={() => insertAtCaret('#')}>
          + #hashtag
        </button>
        <button className="btn-outline text-xs" onClick={() => insertAtCaret('@')}>
          + @mention
        </button>
        <button
          className="btn-outline text-xs"
          onClick={() =>
            insertAtCaret(
              ['#creator', '#content', '#viral', '#fyp'].join(' '),
            )
          }
        >
          + pack tendances
        </button>
      </div>

      <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
        {NETWORKS_FOR_COUNTER.map((n) => {
          const spec = FORMATS[n];
          const over = caption.length > spec.captionMax;
          const hashtagsOver = parsed.hashtags.length > spec.hashtagMax;
          const pct = Math.min(100, (caption.length / spec.captionMax) * 100);
          const active = n === format;
          return (
            <div
              key={n}
              className={`rounded-lg border px-3 py-2 text-xs ${
                active ? 'border-brand-500/60 bg-brand-500/5' : 'border-white/10 bg-white/5'
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="font-medium text-neutral-200">{spec.short}</span>
                <span className={over ? 'text-red-400' : 'text-neutral-400'}>
                  {caption.length}/{spec.captionMax}
                </span>
              </div>
              <div className="mt-1 h-1 overflow-hidden rounded-full bg-white/10">
                <div
                  className={`h-full ${over ? 'bg-red-500' : 'bg-brand-500'}`}
                  style={{ width: `${pct}%` }}
                />
              </div>
              <div
                className={`mt-1 ${
                  hashtagsOver ? 'text-red-400' : 'text-neutral-500'
                }`}
              >
                #: {parsed.hashtags.length}/{spec.hashtagMax}
              </div>
            </div>
          );
        })}
      </div>

      {(parsed.hashtags.length > 0 || parsed.mentions.length > 0) && (
        <div className="space-y-2 rounded-lg border border-white/10 bg-white/5 p-3 text-xs">
          {parsed.hashtags.length > 0 && (
            <div>
              <span className="text-neutral-400">Hashtags détectés : </span>
              {parsed.hashtags.map((h) => (
                <span
                  key={h}
                  className="mr-1 inline-block rounded bg-brand-500/20 px-1.5 py-0.5 text-brand-400"
                >
                  #{h}
                </span>
              ))}
            </div>
          )}
          {parsed.mentions.length > 0 && (
            <div>
              <span className="text-neutral-400">Mentions détectées : </span>
              {parsed.mentions.map((m) => (
                <span
                  key={m}
                  className="mr-1 inline-block rounded bg-sky-500/20 px-1.5 py-0.5 text-sky-400"
                >
                  @{m}
                </span>
              ))}
            </div>
          )}
          <div className="text-[10px] text-neutral-500">
            Longueur max recommandée pour {activeFormatSpec.short} : {activeFormatSpec.captionMax} car.
          </div>
        </div>
      )}
    </div>
  );
}
