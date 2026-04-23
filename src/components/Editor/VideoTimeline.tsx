import { useCallback, useEffect, useRef } from 'react';
import { useEditor } from '../../store/editorStore';

interface Props {
  getVideoElement: () => HTMLVideoElement | null;
}

function formatTime(t: number): string {
  if (!Number.isFinite(t)) return '0:00';
  const m = Math.floor(t / 60);
  const s = Math.floor(t % 60);
  return `${m}:${s.toString().padStart(2, '0')}`;
}

export function VideoTimeline({ getVideoElement }: Props) {
  const media = useEditor((s) => s.media);
  const video = useEditor((s) => s.video);
  const setVideoTime = useEditor((s) => s.setVideoTime);
  const setVideoTrim = useEditor((s) => s.setVideoTrim);
  const trackRef = useRef<HTMLDivElement>(null);

  const isVideo = media?.kind === 'video';
  const duration = isVideo ? media.duration : 0;

  const togglePlay = useCallback(() => {
    const v = getVideoElement();
    if (!v) return;
    if (v.paused) {
      // Clamp to trim start if we're outside the range.
      if (v.currentTime < video.trimStart || v.currentTime > video.trimEnd) {
        v.currentTime = video.trimStart;
      }
      v.play();
    } else {
      v.pause();
    }
  }, [getVideoElement, video.trimStart, video.trimEnd]);

  // Auto-pause when we cross the trim-end while playing.
  useEffect(() => {
    if (!video.playing) return;
    if (video.currentTime >= video.trimEnd) {
      const v = getVideoElement();
      v?.pause();
      if (v) v.currentTime = video.trimEnd;
    }
  }, [video.currentTime, video.playing, video.trimEnd, getVideoElement]);

  if (!isVideo) return null;

  const scrubAt = (clientX: number) => {
    const track = trackRef.current;
    if (!track) return;
    const rect = track.getBoundingClientRect();
    const ratio = Math.min(1, Math.max(0, (clientX - rect.left) / rect.width));
    const t = ratio * duration;
    const v = getVideoElement();
    if (v) v.currentTime = t;
    setVideoTime(t);
  };

  const onScrubDown = (e: React.PointerEvent) => {
    (e.target as Element).setPointerCapture?.(e.pointerId);
    scrubAt(e.clientX);
    const move = (ev: PointerEvent) => scrubAt(ev.clientX);
    const up = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  };

  const onTrimDown = (edge: 'start' | 'end') => (e: React.PointerEvent) => {
    e.stopPropagation();
    const track = trackRef.current;
    if (!track) return;
    const rect = track.getBoundingClientRect();
    const move = (ev: PointerEvent) => {
      const ratio = Math.min(1, Math.max(0, (ev.clientX - rect.left) / rect.width));
      const t = ratio * duration;
      if (edge === 'start') {
        setVideoTrim(Math.min(t, video.trimEnd - 0.1), video.trimEnd);
      } else {
        setVideoTrim(video.trimStart, Math.max(t, video.trimStart + 0.1));
      }
    };
    const up = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  };

  const posPct = duration > 0 ? (video.currentTime / duration) * 100 : 0;
  const startPct = duration > 0 ? (video.trimStart / duration) * 100 : 0;
  const endPct = duration > 0 ? (video.trimEnd / duration) * 100 : 100;

  return (
    <div className="flex items-center gap-3 border-t border-white/10 bg-neutral-950/80 px-4 py-2 backdrop-blur">
      <button className="btn-outline" onClick={togglePlay} title="Lecture / Pause">
        {video.playing ? '⏸' : '▶'}
      </button>
      <div className="flex-1">
        <div
          ref={trackRef}
          className="relative h-7 cursor-pointer select-none rounded bg-neutral-800"
          onPointerDown={onScrubDown}
        >
          {/* Trim range highlight */}
          <div
            className="absolute top-0 h-full rounded bg-brand-500/30"
            style={{ left: `${startPct}%`, width: `${endPct - startPct}%` }}
          />
          {/* Trim handles */}
          <div
            className="absolute top-0 z-20 flex h-full w-2 cursor-ew-resize items-center justify-center rounded-l bg-brand-500"
            style={{ left: `calc(${startPct}% - 4px)` }}
            onPointerDown={onTrimDown('start')}
            title="Début de la coupe"
          >
            <div className="h-3 w-[1px] bg-white/80" />
          </div>
          <div
            className="absolute top-0 z-20 flex h-full w-2 cursor-ew-resize items-center justify-center rounded-r bg-brand-500"
            style={{ left: `calc(${endPct}% - 4px)` }}
            onPointerDown={onTrimDown('end')}
            title="Fin de la coupe"
          >
            <div className="h-3 w-[1px] bg-white/80" />
          </div>
          {/* Playhead */}
          <div
            className="pointer-events-none absolute top-0 z-10 h-full w-[2px] bg-white"
            style={{ left: `${posPct}%` }}
          />
        </div>
      </div>
      <div className="text-xs tabular-nums text-neutral-300">
        {formatTime(video.currentTime)} / {formatTime(duration)}
      </div>
      <div className="text-[10px] text-neutral-500">
        Coupe {formatTime(video.trimEnd - video.trimStart)}
      </div>
    </div>
  );
}
