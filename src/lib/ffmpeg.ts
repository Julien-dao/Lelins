import { FFmpeg } from '@ffmpeg/ffmpeg';

let instance: FFmpeg | null = null;
let loadPromise: Promise<FFmpeg> | null = null;

export type FFmpegProgress = (percent: number, time: number) => void;

/**
 * Charge FFmpeg.wasm à la demande depuis /ffmpeg/ (servi localement).
 * La première invocation télécharge ~32 Mo de WASM, les suivantes sont instantanées.
 */
export async function getFFmpeg(onLog?: (msg: string) => void): Promise<FFmpeg> {
  if (instance) return instance;
  if (loadPromise) return loadPromise;

  loadPromise = (async () => {
    const ff = new FFmpeg();
    if (onLog) ff.on('log', ({ message }) => onLog(message));
    await ff.load({
      coreURL: '/ffmpeg/ffmpeg-core.js',
      wasmURL: '/ffmpeg/ffmpeg-core.wasm',
    });
    instance = ff;
    return ff;
  })();

  try {
    return await loadPromise;
  } catch (err) {
    loadPromise = null;
    throw err;
  }
}

export function isFFmpegLoaded(): boolean {
  return instance !== null;
}

export function onFFmpegProgress(ff: FFmpeg, cb: FFmpegProgress): () => void {
  const handler = ({ progress, time }: { progress: number; time: number }) => {
    cb(Math.min(1, Math.max(0, progress)) * 100, time);
  };
  ff.on('progress', handler);
  return () => ff.off('progress', handler);
}
