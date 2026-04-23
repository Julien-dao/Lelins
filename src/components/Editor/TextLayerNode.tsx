import { useEffect, useRef } from 'react';
import { Group, Rect, Text } from 'react-konva';
import Konva from 'konva';
import { useEditor } from '../../store/editorStore';
import type { TextLayer } from '../../types';

interface Props {
  layer: TextLayer;
}

export function TextLayerNode({ layer }: Props) {
  const selectLayer = useEditor((s) => s.selectLayer);
  const updateLayer = useEditor((s) => s.updateLayer);
  const textRef = useRef<Konva.Text>(null);
  const bgRef = useRef<Konva.Rect>(null);

  const fontStyle = [layer.bold ? 'bold' : '', layer.italic ? 'italic' : '']
    .filter(Boolean)
    .join(' ')
    .trim();

  // Keep background rect sized to the actual rendered text bounds.
  useEffect(() => {
    const text = textRef.current;
    const bg = bgRef.current;
    if (!text || !bg) return;
    if (layer.background === 'none') {
      bg.visible(false);
      return;
    }
    bg.visible(true);
    const padX = 24;
    const padY = 12;
    const w = text.width();
    const h = text.height();
    let x = 0;
    if (layer.align === 'center') x = (layer.width - w) / 2;
    else if (layer.align === 'right') x = layer.width - w;
    bg.position({ x: x - padX, y: -padY });
    bg.size({ width: w + padX * 2, height: h + padY * 2 });
    bg.cornerRadius(layer.background === 'pill' ? (h + padY * 2) / 2 : 8);
  }, [layer.text, layer.fontSize, layer.fontFamily, layer.background, layer.align, layer.width, layer.bold, layer.italic]);

  return (
    <Group
      id={layer.id}
      x={layer.x}
      y={layer.y}
      rotation={layer.rotation}
      opacity={layer.opacity}
      draggable
      onClick={() => selectLayer(layer.id)}
      onTap={() => selectLayer(layer.id)}
      onDragEnd={(e) => updateLayer(layer.id, { x: e.target.x(), y: e.target.y() })}
      onTransformEnd={(e) => {
        const node = e.target as Konva.Group;
        const scaleX = node.scaleX();
        node.scaleX(1);
        node.scaleY(1);
        updateLayer(layer.id, {
          x: node.x(),
          y: node.y(),
          rotation: node.rotation(),
          width: Math.max(40, layer.width * scaleX),
        });
      }}
      onDblClick={() => {
        const next = window.prompt('Modifier le texte', layer.text);
        if (next !== null) updateLayer(layer.id, { text: next });
      }}
      onDblTap={() => {
        const next = window.prompt('Modifier le texte', layer.text);
        if (next !== null) updateLayer(layer.id, { text: next });
      }}
    >
      <Rect
        ref={bgRef}
        fill={layer.backgroundColor}
        opacity={layer.background === 'none' ? 0 : 0.7}
        listening={false}
      />
      <Text
        ref={textRef}
        text={layer.text}
        width={layer.width}
        align={layer.align}
        fontSize={layer.fontSize}
        fontFamily={layer.fontFamily}
        fontStyle={fontStyle || 'normal'}
        fill={layer.color}
        stroke={layer.strokeWidth > 0 ? layer.stroke : undefined}
        strokeWidth={layer.strokeWidth}
        lineHeight={1.2}
        listening={true}
      />
    </Group>
  );
}
