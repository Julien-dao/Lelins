import { fetchFile } from '@ffmpeg/util';
import { getFFmpeg, onFFmpegProgress } from './ffmpeg';
import { buildAss } from './subtitles';
import type { FilterState, SubtitleSegment, SubtitleStyle, VideoSource } from '../types';

export interface VideoExportOptions {
  source: VideoSource;
  trimStart: number;
  trimEnd: number;
  width: number;
  height: number;
  /** PNG overlay (text + sous-titres statiques) avec fond transparent. */
  overlayPng: Uint8Array | null;
  /** Sous-titres timés à brûler via filter 'subtitles' (séparés des layers statiques). */
  subtitleTrack: SubtitleSegment[];
  subtitleStyle: SubtitleStyle;
  filter: FilterState;
  onProgress?: (percent: number) => void;
  onLog?: (line: string) => void;
}

interface FilterGraph {
  graph: string;
  hasOverlay: boolean;
}

function buildFilterGraph(
  width: number,
  height: number,
  f: FilterState,
  hasOverlay: boolean,
  hasSubtitles: boolean,
  subtitlePath: string,
): FilterGraph {
  const eq: string[] = [];
  eq.push(`brightness=${f.brightness.toFixed(3)}`);
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
  parts.push(
    `scale=${width}:${height}:force_original_aspect_ratio=increase,crop=${width}:${height}`,
  );

  // Build chain: [0:v]<filters>[v0]; [v0]<subtitles>[v1]; [v1][1:v]overlay[v]
  const segments: string[] = [];
  segments.push(`[0:v]${parts.join(',')}[v0]`);
  let last = 'v0';

  if (hasSubtitles) {
    // Escape ':' and ',' in subtitle path for FFmpeg filter syntax.
    const safe = subtitlePath.replace(/'/g, "\\'");
    segments.push(`[${last}]subtitles='${safe}'[v1]`);
    last = 'v1';
  }

  if (hasOverlay) {
    segments.push(`[${last}][1:v]overlay=0:0:format=auto[v]`);
    return { graph: segments.join(';'), hasOverlay: true };
  }

  // Rename last to [v] for the -map.
  segments[segments.length - 1] = segments[segments.length - 1].replace(`[${last}]`, '[v]');
  return { graph: segments.join(';'), hasOverlay: false };
}

export async function exportVideo(opts: VideoExportOptions): Promise<Blob> {
  const ff = await getFFmpeg(opts.onLog);
  const releaseProgress = opts.onProgress
    ? onFFmpegProgress(ff, (p) => opts.onProgress!(p))
    : () => {};

  const inputName = 'input' + guessExt(opts.source.file.name);
  const overlayName = 'overlay.png';
  const subtitleName = 'subs.ass';
  const outputName = 'output.mp4';

  const hasSubtitles = opts.subtitleTrack.length > 0;
  const hasOverlay = opts.overlayPng !== null;

  try {
    await ff.writeFile(inputName, await fetchFile(opts.source.file));
    if (hasOverlay) {
      await ff.writeFile(overlayName, opts.overlayPng!);
    }
    if (hasSubtitles) {
      const ass = buildAss(
        opts.subtitleTrack,
        opts.subtitleStyle,
        opts.width,
        opts.height,
        opts.trimStart,
      );
      await ff.writeFile(subtitleName, new TextEncoder().encode(ass));
    }

    const { graph } = buildFilterGraph(
      opts.width,
      opts.height,
      opts.filter,
      hasOverlay,
      hasSubtitles,
      subtitleName,
    );

    const args: string[] = [
      '-ss',
      opts.trimStart.toFixed(3),
      '-to',
      opts.trimEnd.toFixed(3),
      '-i',
      inputName,
    ];
    if (hasOverlay) args.push('-i', overlayName);
    args.push(
      '-filter_complex',
      graph,
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
    const bytes = data as Uint8Array;
    const copy = new Uint8Array(bytes.byteLength);
    copy.set(bytes);
    return new Blob([copy], { type: 'video/mp4' });
  } finally {
    releaseProgress();
    for (const name of [inputName, overlayName, subtitleName, outputName]) {
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
