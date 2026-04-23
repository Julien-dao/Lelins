import Konva from 'konva';
import type { TextLayer } from '../types';

/**
 * Render only the text layers onto a transparent canvas at native format dimensions.
 * Used as the "overlay" stream when exporting video via FFmpeg.
 */
export async function renderOverlayPng(
  layers: TextLayer[],
  width: number,
  height: number,
): Promise<Uint8Array | null> {
  if (layers.length === 0) return null;

  const container = document.createElement('div');
  container.style.position = 'fixed';
  container.style.left = '-99999px';
  container.style.top = '-99999px';
  document.body.appendChild(container);

  try {
    const stage = new Konva.Stage({ container, width, height });
    const layer = new Konva.Layer();
    stage.add(layer);

    for (const l of layers) {
      const group = new Konva.Group({
        x: l.x,
        y: l.y,
        rotation: l.rotation,
        opacity: l.opacity,
      });
      const tempText = new Konva.Text({
        text: l.text,
        width: l.width,
        align: l.align,
        fontSize: l.fontSize,
        fontFamily: l.fontFamily,
        fontStyle: [l.bold ? 'bold' : '', l.italic ? 'italic' : '']
          .filter(Boolean)
          .join(' ')
          .trim() || 'normal',
        fill: l.color,
        stroke: l.strokeWidth > 0 ? l.stroke : undefined,
        strokeWidth: l.strokeWidth,
        lineHeight: 1.2,
      });

      if (l.background !== 'none') {
        const w = tempText.width();
        const h = tempText.height();
        const padX = 24;
        const padY = 12;
        let x = 0;
        if (l.align === 'center') x = (l.width - w) / 2;
        else if (l.align === 'right') x = l.width - w;
        const bg = new Konva.Rect({
          x: x - padX,
          y: -padY,
          width: w + padX * 2,
          height: h + padY * 2,
          fill: l.backgroundColor,
          opacity: 0.7,
          cornerRadius: l.background === 'pill' ? (h + padY * 2) / 2 : 8,
        });
        group.add(bg);
      }

      group.add(tempText);
      layer.add(group);
    }

    stage.draw();

    const dataUrl = stage.toDataURL({ mimeType: 'image/png', pixelRatio: 1 });
    stage.destroy();

    const bytes = dataUrlToUint8(dataUrl);
    return bytes;
  } finally {
    container.remove();
  }
}

function dataUrlToUint8(dataUrl: string): Uint8Array {
  const comma = dataUrl.indexOf(',');
  const b64 = dataUrl.slice(comma + 1);
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}
