export interface ParsedCaption {
  hashtags: string[];
  mentions: string[];
  body: string;
}

const HASHTAG_RE = /(^|\s)#([\p{L}\p{N}_]+)/gu;
const MENTION_RE = /(^|\s)@([\p{L}\p{N}_.]+)/gu;

export function parseCaption(text: string): ParsedCaption {
  const hashtags: string[] = [];
  const mentions: string[] = [];
  for (const m of text.matchAll(HASHTAG_RE)) hashtags.push(m[2]);
  for (const m of text.matchAll(MENTION_RE)) mentions.push(m[2]);
  return {
    hashtags: Array.from(new Set(hashtags)),
    mentions: Array.from(new Set(mentions)),
    body: text,
  };
}

/**
 * Returns the token currently being typed at `caretPos`, or null.
 * Example: "Hello #par|" → { kind: '#', value: 'par', start: 6, end: 10 }
 */
export function getActiveToken(
  text: string,
  caretPos: number,
): { kind: '#' | '@'; value: string; start: number; end: number } | null {
  let i = caretPos - 1;
  while (i >= 0 && /[\p{L}\p{N}_.]/u.test(text[i])) i--;
  if (i < 0) return null;
  const trigger = text[i];
  if (trigger !== '#' && trigger !== '@') return null;
  const before = i === 0 ? ' ' : text[i - 1];
  if (!/\s/.test(before)) return null;
  return {
    kind: trigger,
    value: text.slice(i + 1, caretPos),
    start: i,
    end: caretPos,
  };
}

export function replaceToken(
  text: string,
  token: { start: number; end: number; kind: '#' | '@' },
  replacement: string,
): { text: string; caret: number } {
  const before = text.slice(0, token.start);
  const after = text.slice(token.end);
  const inserted = `${token.kind}${replacement} `;
  return {
    text: before + inserted + after,
    caret: before.length + inserted.length,
  };
}

export const SUGGESTED_HASHTAGS = [
  'creator',
  'contentcreator',
  'design',
  'reels',
  'viral',
  'fyp',
  'pourtoi',
  'photooftheday',
  'love',
  'instagood',
  'style',
  'mode',
  'branding',
  'marketing',
  'business',
  'entrepreneur',
  'inspiration',
  'motivation',
];
