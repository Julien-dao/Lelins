import { useEffect, useMemo, useRef } from 'react';
import { Group, Rect, Text } from 'react-konva';
import Konva from 'konva';
import { useEditor } from '../../store/editorStore';
import { getFormat } from '../../lib/formats';

/**
 * Renders the auto-generated subtitle segment whose time range covers `currentTime`.
 * Style is shared across all segments (driven by subtitleStyle in the store).
 */
export function LiveSubtitleNode() {
  const segments = useEditor((s) => s.subtitleTrack);
  const style = useEditor((s) => s.subtitleStyle);
  const format = useEditor((s) => s.format);
  const currentTime = useEditor((s) => s.video.currentTime);

  const spec = getFormat(format);
  const textRef = useRef<Konva.Text>(null);
  const bgRef = useRef<Konva.Rect>(null);

  const active = useMemo(
    () => segments.find((s) => currentTime >= s.start && currentTime <= s.end),
    [segments, currentTime],
  );

  // Resize the background to fit the rendered text.
  useEffect(() => {
    const text = textRef.current;
    const bg = bgRef.current;
    if (!text || !bg || !active) return;
    const padX = 32;
    const padY = 16;
    const w = text.width();
    const h = text.height();
    const totalW = spec.width * 0.85;
    const x = (totalW - w) / 2;
    bg.position({ x: x - padX, y: -padY });
    bg.size({ width: w + padX * 2, height: h + padY * 2 });
    bg.cornerRadius(style.background === 'pill' ? (h + padY * 2) / 2 : 12);
  }, [active, style, spec.width]);

  if (!active) return null;

  const groupY =
    style.position === 'top'
      ? spec.height * 0.1
      : style.position === 'middle'
        ? spec.height / 2 - style.fontSize
        : spec.height * 0.78;

  const groupX = spec.width * 0.075;
  const totalW = spec.width * 0.85;

  return (
    <Group x={groupX} y={groupY} listening={false}>
      <Rect
        ref={bgRef}
        fill={style.backgroundColor}
        opacity={style.background === 'none' ? 0 : 0.7}
      />
      <Text
        ref={textRef}
        text={active.text}
        width={totalW}
        align="center"
        fontSize={style.fontSize}
        fontFamily={style.fontFamily}
        fontStyle="bold"
        fill={style.color}
        stroke="#000"
        strokeWidth={1.5}
        lineHeight={1.2}
      />
    </Group>
  );
}
