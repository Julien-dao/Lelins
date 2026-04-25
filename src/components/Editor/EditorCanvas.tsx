import { forwardRef, useEffect, useImperativeHandle, useMemo, useRef } from 'react';
import { Image as KImage, Layer, Rect, Stage, Transformer } from 'react-konva';
import Konva from 'konva';
import useImage from 'use-image';
import { useEditor } from '../../store/editorStore';
import { getFormat } from '../../lib/formats';
import { TextLayerNode } from './TextLayerNode';
import { LiveSubtitleNode } from './LiveSubtitleNode';

export interface EditorCanvasHandle {
  toDataURL: (mimeType?: string, quality?: number) => string | null;
  toBlob: (mimeType?: string, quality?: number) => Promise<Blob | null>;
  getVideoElement: () => HTMLVideoElement | null;
}

interface Props {
  containerWidth: number;
  containerHeight: number;
}

export const EditorCanvas = forwardRef<EditorCanvasHandle, Props>(function EditorCanvas(
  { containerWidth, containerHeight },
  ref,
) {
  const media = useEditor((s) => s.media);
  const format = useEditor((s) => s.format);
  const layers = useEditor((s) => s.layers);
  const filter = useEditor((s) => s.filter);
  const selectedLayerId = useEditor((s) => s.selectedLayerId);
  const selectLayer = useEditor((s) => s.selectLayer);
  const videoState = useEditor((s) => s.video);
  const setVideoTime = useEditor((s) => s.setVideoTime);
  const setVideoPlaying = useEditor((s) => s.setVideoPlaying);

  const spec = getFormat(format);
  const stageRef = useRef<Konva.Stage>(null);
  const bgImageRef = useRef<Konva.Image>(null);
  const transformerRef = useRef<Konva.Transformer>(null);
  const videoElRef = useRef<HTMLVideoElement | null>(null);
  const rafRef = useRef<number | null>(null);

  const imageSrc = media?.kind === 'image' ? media.src : '';
  const [loadedImage] = useImage(imageSrc, 'anonymous');

  // Instantiate / dispose the background <video> element for video sources.
  useEffect(() => {
    if (media?.kind !== 'video') {
      videoElRef.current?.pause();
      videoElRef.current = null;
      return;
    }
    const v = document.createElement('video');
    v.src = media.src;
    v.crossOrigin = 'anonymous';
    v.muted = true;
    v.playsInline = true;
    v.preload = 'auto';
    v.addEventListener('loadeddata', () => {
      v.currentTime = 0;
      bgImageRef.current?.getLayer()?.batchDraw();
    });
    v.addEventListener('timeupdate', () => {
      setVideoTime(v.currentTime);
    });
    v.addEventListener('play', () => setVideoPlaying(true));
    v.addEventListener('pause', () => setVideoPlaying(false));
    videoElRef.current = v;
    return () => {
      v.pause();
      v.src = '';
    };
  }, [media, setVideoTime, setVideoPlaying]);

  // While the video plays, redraw the Konva layer each animation frame.
  useEffect(() => {
    const v = videoElRef.current;
    if (!v) return;
    const tick = () => {
      bgImageRef.current?.getLayer()?.batchDraw();
      rafRef.current = requestAnimationFrame(tick);
    };
    if (videoState.playing) {
      rafRef.current = requestAnimationFrame(tick);
    }
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
    };
  }, [videoState.playing]);

  // Sync the background image source (image OR video element) to Konva.
  useEffect(() => {
    const node = bgImageRef.current;
    if (!node) return;
    const v = videoElRef.current;
    if (media?.kind === 'video' && v) {
      node.image(v);
      node.getLayer()?.batchDraw();
    } else if (loadedImage) {
      node.image(loadedImage);
      node.getLayer()?.batchDraw();
    }
  }, [loadedImage, media]);

  // Fit canvas into the container.
  const scale = useMemo(() => {
    const padding = 32;
    const availW = Math.max(100, containerWidth - padding);
    const availH = Math.max(100, containerHeight - padding);
    return Math.min(availW / spec.width, availH / spec.height);
  }, [containerWidth, containerHeight, spec.width, spec.height]);

  // Apply Konva filters to the background node.
  useEffect(() => {
    const node = bgImageRef.current;
    if (!node || !node.image()) return;
    // For video, re-cache on each frame is too expensive.
    // We only cache for images; videos apply filters via CSS on export.
    if (media?.kind === 'image') {
      node.cache();
    } else {
      // Ensure no cache so the live video updates properly.
      try {
        node.clearCache();
      } catch {
        /* ignore */
      }
    }
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
  }, [loadedImage, filter, media]);

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
      const t = transformerRef.current;
      t?.nodes([]);
      stage.batchDraw();
      const url = stage.toDataURL({
        mimeType,
        quality,
        pixelRatio: 1 / scale,
      });
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
    getVideoElement: () => videoElRef.current,
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
            <KImage
              ref={bgImageRef}
              image={undefined}
              x={0}
              y={0}
              width={spec.width}
              height={spec.height}
              listening={false}
            />
          </Layer>
          <Layer>
            {layers.map((layer) => (
              <TextLayerNode key={layer.id} layer={layer} />
            ))}
            <LiveSubtitleNode />
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
        {!media && (
          <div className="pointer-events-none absolute inset-0 flex items-center justify-center text-center text-sm text-neutral-400">
            <div>
              <p className="text-lg font-semibold text-neutral-200">Commencez ici</p>
              <p className="mt-1">Importez une image ou une vidéo pour démarrer.</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
});
