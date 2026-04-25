import type { SubtitleSegment, SubtitleStyle } from '../types';

function fmtAssTime(t: number): string {
  if (t < 0) t = 0;
  const h = Math.floor(t / 3600);
  const m = Math.floor((t % 3600) / 60);
  const s = Math.floor(t % 60);
  const cs = Math.floor((t * 100) % 100);
  return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}.${cs.toString().padStart(2, '0')}`;
}

/** RGB hex (#rrggbb) → ASS &HBBGGRR& with fully opaque alpha. */
function hexToAss(hex: string, alpha: number = 0): string {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex);
  const a = Math.round(Math.min(255, Math.max(0, alpha))).toString(16).padStart(2, '0');
  if (!m) return `&H${a}FFFFFF`;
  const rr = m[1].slice(0, 2);
  const gg = m[1].slice(2, 4);
  const bb = m[1].slice(4, 6);
  return `&H${a}${bb}${gg}${rr}`.toUpperCase();
}

function escapeText(text: string): string {
  return text.replace(/\r?\n/g, '\\N').replace(/\{/g, '(').replace(/\}/g, ')');
}

/**
 * Builds an ASS file mapping our SubtitleStyle/segments. Time-shifted by `offset`
 * so that exporting a trimmed range starts at 0.
 */
export function buildAss(
  segments: SubtitleSegment[],
  style: SubtitleStyle,
  videoWidth: number,
  videoHeight: number,
  offset: number = 0,
): string {
  // ASS Alignment: 8=top center, 5=middle center, 2=bottom center
  const alignment =
    style.position === 'top' ? 8 : style.position === 'middle' ? 5 : 2;

  // ASS BorderStyle: 1 = outline + shadow, 3 = opaque box
  const borderStyle = style.background === 'none' ? 1 : 3;
  const marginV = Math.round(videoHeight * 0.08);

  const primary = hexToAss(style.color);
  const back =
    style.background === 'none'
      ? hexToAss('#000000', 80)
      : hexToAss(style.backgroundColor, 80);
  const outline = hexToAss('#000000');

  const header = [
    '[Script Info]',
    'ScriptType: v4.00+',
    'PlayResX: ' + videoWidth,
    'PlayResY: ' + videoHeight,
    'WrapStyle: 0',
    'ScaledBorderAndShadow: yes',
    '',
    '[V4+ Styles]',
    'Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding',
    `Style: Default,${style.fontFamily},${style.fontSize},${primary},&H000000FF,${outline},${back},-1,0,0,0,100,100,0,0,${borderStyle},2,0,${alignment},40,40,${marginV},1`,
    '',
    '[Events]',
    'Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text',
  ];

  const events = segments
    .filter((seg) => seg.end > offset)
    .map((seg) => {
      const start = Math.max(0, seg.start - offset);
      const end = Math.max(start + 0.05, seg.end - offset);
      return `Dialogue: 0,${fmtAssTime(start)},${fmtAssTime(end)},Default,,0,0,0,,${escapeText(seg.text)}`;
    });

  return [...header, ...events].join('\n');
}
