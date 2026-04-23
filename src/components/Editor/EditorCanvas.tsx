import { forwardRef, useEffect, useImperativeHandle, useMemo, useRef } from 'react';
import { Image as KImage, Layer, Rect, Stage, Transformer } from 'react-konva';
import Konva from 'konva';
import useImage from 'use-image';
import { useEditor } from '../../store/editorStore';
import { getFormat } from '../../lib/formats';
import { TextLayerNode } from './TextLayerNode';

export interface EditorCanvasHandle {
  toDataURL: (mimeType?: string, quality?: number) => string | null;
  toBlob: (mimeType?: string, quality?: number) => Promise<Blob | null>;
}

interface Props {
  containerWidth: number;
  containerHeight: number;
}

export const EditorCanvas = forwardRef<EditorCanvasHandle, Props>(function EditorCanvas(
  { containerWidth, containerHeight },
  ref,
) {
  const image = useEditor((s) => s.image);
  const format = useEditor((s) => s.format);
  const layers = useEditor((s) => s.layers);
  const filter = useEditor((s) => s.filter);
  const selectedLayerId = useEditor((s) => s.selectedLayerId);
  const selectLayer = useEditor((s) => s.selectLayer);

  const spec = getFormat(format);
  const stageRef = useRef<Konva.Stage>(null);
  const bgImageRef = useRef<Konva.Image>(null);
  const transformerRef = useRef<Konva.Transformer>(null);
  const [loadedImage] = useImage(image?.src ?? '', 'anonymous');

  // Fit the stage into the container while preserving the canvas aspect.
  const scale = useMemo(() => {
    const padding = 32;
    const availW = Math.max(100, containerWidth - padding);
    const availH = Math.max(100, containerHeight - padding);
    return Math.min(availW / spec.width, availH / spec.height);
  }, [containerWidth, containerHeight, spec.width, spec.height]);

  // Attach the Konva filter pipeline to the background image node whenever it changes.
  useEffect(() => {
    const node = bgImageRef.current;
    if (!node || !loadedImage) return;
    node.cache();
    node.filters([
      Konva.Filters.Brighten,
      Konva.Filters.Contrast,
      Konva.Filters.HSL,
      Konva.Filters.Blur,
      ...(filter.grayscale ? [Konva.Filters.Grayscale] : []),
      ...(filter.sepia ? [Konva.Filters.Sepia] : []),
      ...(filter.invert ? [Konva.Filters.Invert] : []),
    ]);
    node.brightness(filter.brightness);
    node.contrast(filter.contrast);
    node.saturation(filter.saturation);
    node.hue(filter.hue);
    node.blurRadius(filter.blur);
    node.getLayer()?.batchDraw();
  }, [loadedImage, filter]);

  // Keep Transformer attached to the selected layer node.
  useEffect(() => {
    const transformer = transformerRef.current;
    const stage = stageRef.current;
    if (!transformer || !stage) return;
    if (!selectedLayerId) {
      transformer.nodes([]);
      transformer.getLayer()?.batchDraw();
      return;
    }
    const selected = stage.findOne(`#${selectedLayerId}`);
    if (selected) {
      transformer.nodes([selected]);
      transformer.getLayer()?.batchDraw();
    } else {
      transformer.nodes([]);
    }
  }, [selectedLayerId, layers]);

  useImperativeHandle(ref, () => ({
    toDataURL: (mimeType = 'image/png', quality = 0.95) => {
      const stage = stageRef.current;
      if (!stage) return null;
      // Temporarily clear selection so the transformer doesn't bleed into export.
      const t = transformerRef.current;
      t?.nodes([]);
      stage.batchDraw();
      const url = stage.toDataURL({
        mimeType,
        quality,
        pixelRatio: 1 / scale, // export at native format resolution
      });
      // Restore selection after export.
      if (selectedLayerId) {
        const selected = stage.findOne(`#${selectedLayerId}`);
        if (selected && t) t.nodes([selected]);
      }
      stage.batchDraw();
      return url;
    },
    toBlob: async (mimeType = 'image/png', quality = 0.95) => {
      const stage = stageRef.current;
      if (!stage) return null;
      const t = transformerRef.current;
      t?.nodes([]);
      stage.batchDraw();
      const blob = await new Promise<Blob | null>((resolve) => {
        stage.toCanvas({ pixelRatio: 1 / scale }).toBlob(
          (b) => resolve(b),
          mimeType,
          quality,
        );
      });
      if (selectedLayerId) {
        const selected = stage.findOne(`#${selectedLayerId}`);
        if (selected && t) t.nodes([selected]);
      }
      stage.batchDraw();
      return blob;
    },
  }));

  const stageW = spec.width * scale;
  const stageH = spec.height * scale;

  return (
    <div
      className="flex items-center justify-center"
      style={{ width: containerWidth, height: containerHeight }}
    >
      <div
        className="relative shadow-2xl ring-1 ring-white/10"
        style={{ width: stageW, height: stageH }}
      >
        <Stage
          ref={stageRef}
          width={stageW}
          height={stageH}
          scale={{ x: scale, y: scale }}
          onMouseDown={(e) => {
            if (e.target === e.target.getStage()) selectLayer(null);
          }}
          onTap={(e) => {
            if (e.target === e.target.getStage()) selectLayer(null);
          }}
        >
          <Layer>
            <Rect x={0} y={0} width={spec.width} height={spec.height} fill="#111" />
            {loadedImage && (
              <KImage
                ref={bgImageRef}
                image={loadedImage}
                x={0}
                y={0}
                width={spec.width}
                height={spec.height}
                listening={false}
              />
            )}
          </Layer>
          <Layer>
            {layers.map((layer) => (
              <TextLayerNode key={layer.id} layer={layer} />
            ))}
            <Transformer
              ref={transformerRef}
              rotateEnabled
              enabledAnchors={['middle-left', 'middle-right', 'top-center', 'bottom-center']}
              anchorStroke="#8b5cf6"
              anchorFill="#fff"
              borderStroke="#8b5cf6"
              borderDash={[4, 4]}
              boundBoxFunc={(oldBox, newBox) => {
                if (newBox.width < 40) return oldBox;
                return newBox;
              }}
            />
          </Layer>
        </Stage>
        {!image && (
          <div className="pointer-events-none absolute inset-0 flex items-center justify-center text-center text-sm text-neutral-400">
            <div>
              <p className="text-lg font-semibold text-neutral-200">Commencez ici</p>
              <p className="mt-1">Importez une image pour démarrer.</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
});
