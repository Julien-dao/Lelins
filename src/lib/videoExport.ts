import { fetchFile } from '@ffmpeg/util';
import { getFFmpeg, onFFmpegProgress } from './ffmpeg';
import type { FilterState, VideoSource } from '../types';

export interface VideoExportOptions {
  source: VideoSource;
  trimStart: number;
  trimEnd: number;
  /** Target width in px of the exported MP4. Height preserved from source crop. */
  width: number;
  height: number;
  /** PNG overlay (text + subtitles) with transparent background, sized width×height. */
  overlayPng: Uint8Array | null;
  filter: FilterState;
  onProgress?: (percent: number) => void;
  onLog?: (line: string) => void;
}

/**
 * Build the FFmpeg -vf graph to apply filter + scale/pad to target dims + overlay PNG.
 * Chained on the decoded video stream [0:v].
 */
function buildVideoFilter(
  width: number,
  height: number,
  f: FilterState,
  hasOverlay: boolean,
): { filter: string; useOverlayInput: boolean } {
  const eq: string[] = [];
  // FFmpeg `eq` filter: brightness (-1..1), contrast (0..n, 1=neutral), saturation, gamma
  eq.push(`brightness=${(f.brightness).toFixed(3)}`);
  eq.push(`contrast=${(1 + f.contrast / 100).toFixed(3)}`);
  eq.push(`saturation=${(1 + f.saturation / 2).toFixed(3)}`);

  const parts: string[] = [];
  parts.push(`eq=${eq.join(':')}`);
  if (f.hue !== 0) parts.push(`hue=h=${f.hue}`);
  if (f.blur > 0) parts.push(`boxblur=${Math.max(1, Math.round(f.blur / 2))}:1`);
  if (f.grayscale) parts.push('format=gray,format=yuv420p');
  if (f.sepia) {
    parts.push(
      'colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131',
    );
  }
  if (f.invert) parts.push('negate');

  // Cover-fit: scale then pad to exact format dims.
  parts.push(
    `scale=${width}:${height}:force_original_aspect_ratio=increase,crop=${width}:${height}`,
  );

  const base = parts.join(',');

  if (hasOverlay) {
    // [0:v]<filters>[v0]; [v0][1:v]overlay=0:0[v]
    return {
      filter: `[0:v]${base}[v0];[v0][1:v]overlay=0:0:format=auto[v]`,
      useOverlayInput: true,
    };
  }
  return { filter: `[0:v]${base}[v]`, useOverlayInput: false };
}

export async function exportVideo(opts: VideoExportOptions): Promise<Blob> {
  const ff = await getFFmpeg(opts.onLog);
  const releaseProgress = opts.onProgress
    ? onFFmpegProgress(ff, (p) => opts.onProgress!(p))
    : () => {};

  const inputName = 'input' + guessExt(opts.source.file.name);
  const overlayName = 'overlay.png';
  const outputName = 'output.mp4';

  try {
    await ff.writeFile(inputName, await fetchFile(opts.source.file));
    if (opts.overlayPng) {
      await ff.writeFile(overlayName, opts.overlayPng);
    }

    const { filter, useOverlayInput } = buildVideoFilter(
      opts.width,
      opts.height,
      opts.filter,
      opts.overlayPng !== null,
    );

    const args: string[] = [
      '-ss',
      opts.trimStart.toFixed(3),
      '-to',
      opts.trimEnd.toFixed(3),
      '-i',
      inputName,
    ];
    if (useOverlayInput) {
      args.push('-i', overlayName);
    }
    args.push(
      '-filter_complex',
      filter,
      '-map',
      '[v]',
      '-map',
      '0:a?',
      '-c:v',
      'libx264',
      '-pix_fmt',
      'yuv420p',
      '-preset',
      'ultrafast',
      '-crf',
      '23',
      '-c:a',
      'aac',
      '-b:a',
      '128k',
      '-movflags',
      '+faststart',
      '-y',
      outputName,
    );

    await ff.exec(args);
    const data = await ff.readFile(outputName);
    // `readFile` renvoie Uint8Array en mode binaire (notre cas).
    const bytes = data as Uint8Array;
    // Copie pour détacher le buffer sous-jacent (peut être partagé avec WASM).
    const copy = new Uint8Array(bytes.byteLength);
    copy.set(bytes);
    return new Blob([copy], { type: 'video/mp4' });
  } finally {
    releaseProgress();
    // Best-effort cleanup — ignore if files aren't present.
    for (const name of [inputName, overlayName, outputName]) {
      try {
        await ff.deleteFile(name);
      } catch {
        /* ignore */
      }
    }
  }
}

function guessExt(filename: string): string {
  const m = filename.match(/\.[a-z0-9]+$/i);
  return m ? m[0] : '.mp4';
}
