import { pipeline, env } from '@huggingface/transformers';
import type { AutomaticSpeechRecognitionPipeline } from '@huggingface/transformers';

// On garde les modèles en cache navigateur (IndexedDB) — premier chargement
// uniquement, ~75 Mo pour le modèle "tiny".
env.allowRemoteModels = true;
// Permet aux utilisateurs de placer des modèles dans /public/models/<repo>
// pour un fonctionnement entièrement hors-ligne après installation.
env.allowLocalModels = true;
env.localModelPath = '/models/';

let pipelinePromise: Promise<AutomaticSpeechRecognitionPipeline> | null = null;

export type WhisperModel =
  | 'Xenova/whisper-tiny'
  | 'Xenova/whisper-base'
  | 'Xenova/whisper-small';

export interface TranscribeProgress {
  status: 'loading' | 'transcribing' | 'done';
  detail?: string;
  percent?: number;
}

interface ProgressCallback {
  status?: string;
  name?: string;
  file?: string;
  progress?: number;
  loaded?: number;
  total?: number;
}

export async function getWhisperPipeline(
  model: WhisperModel = 'Xenova/whisper-tiny',
  onProgress?: (p: TranscribeProgress) => void,
): Promise<AutomaticSpeechRecognitionPipeline> {
  if (pipelinePromise) return pipelinePromise;

  pipelinePromise = (async () => {
    const pipe = await pipeline('automatic-speech-recognition', model, {
      progress_callback: (raw: unknown) => {
        const data = raw as ProgressCallback;
        if (!onProgress) return;
        if (data.status === 'progress' && typeof data.progress === 'number') {
          onProgress({
            status: 'loading',
            detail: `${data.file ?? data.name ?? 'modèle'} (${Math.round(data.progress)}%)`,
            percent: data.progress,
          });
        } else if (data.status === 'ready') {
          onProgress({ status: 'loading', detail: 'modèle prêt', percent: 100 });
        } else if (data.status) {
          onProgress({ status: 'loading', detail: data.status });
        }
      },
    });
    return pipe as AutomaticSpeechRecognitionPipeline;
  })();

  try {
    return await pipelinePromise;
  } catch (err) {
    pipelinePromise = null;
    throw err;
  }
}

export interface WhisperSegment {
  start: number;
  end: number;
  text: string;
}

export async function transcribe(
  audio: Float32Array,
  options: {
    language?: string;
    onProgress?: (p: TranscribeProgress) => void;
    model?: WhisperModel;
  } = {},
): Promise<WhisperSegment[]> {
  const { onProgress, language, model } = options;
  const pipe = await getWhisperPipeline(model, onProgress);
  onProgress?.({ status: 'transcribing', detail: 'analyse audio…' });

  const result = await pipe(audio, {
    chunk_length_s: 30,
    stride_length_s: 5,
    return_timestamps: true,
    language: language || undefined,
    task: 'transcribe',
  });

  // The pipeline returns `{ text, chunks: [{ timestamp: [start,end], text }, ...] }`
  // when return_timestamps is true.
  const single = Array.isArray(result) ? result[0] : result;
  const chunks = (single as { chunks?: Array<{ timestamp: [number, number]; text: string }> })
    .chunks;
  if (!chunks) {
    return [
      {
        start: 0,
        end: 0,
        text: (single as { text: string }).text.trim(),
      },
    ];
  }
  return chunks
    .filter((c) => c.text.trim().length > 0)
    .map((c) => ({
      start: c.timestamp[0] ?? 0,
      end: c.timestamp[1] ?? c.timestamp[0] ?? 0,
      text: c.text.trim(),
    }));
}
