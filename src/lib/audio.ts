import { fetchFile } from '@ffmpeg/util';
import { getFFmpeg } from './ffmpeg';

const TARGET_SAMPLE_RATE = 16000;

/**
 * Extract mono audio from any media file at 16kHz (required by Whisper)
 * and return it as a Float32Array.
 */
export async function extractAudio(file: File): Promise<Float32Array> {
  const ff = await getFFmpeg();
  const ext = (file.name.match(/\.[a-z0-9]+$/i)?.[0] ?? '.mp4').toLowerCase();
  const inputName = `input${ext}`;
  const outputName = 'audio.wav';

  try {
    await ff.writeFile(inputName, await fetchFile(file));
    await ff.exec([
      '-i',
      inputName,
      '-vn',
      '-ac',
      '1',
      '-ar',
      String(TARGET_SAMPLE_RATE),
      '-f',
      'wav',
      '-y',
      outputName,
    ]);
    const data = await ff.readFile(outputName);
    const bytes = data as Uint8Array;
    return decodeWav(bytes);
  } finally {
    for (const name of [inputName, outputName]) {
      try {
        await ff.deleteFile(name);
      } catch {
        /* ignore */
      }
    }
  }
}

/**
 * Decode a 16-bit PCM mono WAV at any sample rate to Float32Array.
 * Whisper expects samples in [-1, 1].
 */
function decodeWav(bytes: Uint8Array): Float32Array {
  // WAV header parsing — find "data" chunk.
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  if (
    String.fromCharCode(view.getUint8(0), view.getUint8(1), view.getUint8(2), view.getUint8(3)) !==
    'RIFF'
  ) {
    throw new Error('Format WAV invalide.');
  }

  let offset = 12;
  let dataOffset = -1;
  let dataLength = 0;
  let bitsPerSample = 16;
  while (offset < bytes.length - 8) {
    const id = String.fromCharCode(
      view.getUint8(offset),
      view.getUint8(offset + 1),
      view.getUint8(offset + 2),
      view.getUint8(offset + 3),
    );
    const size = view.getUint32(offset + 4, true);
    if (id === 'fmt ') {
      bitsPerSample = view.getUint16(offset + 8 + 14, true);
    } else if (id === 'data') {
      dataOffset = offset + 8;
      dataLength = size;
      break;
    }
    offset += 8 + size;
  }
  if (dataOffset < 0) throw new Error('Section data WAV introuvable.');

  if (bitsPerSample === 16) {
    const sampleCount = dataLength / 2;
    const out = new Float32Array(sampleCount);
    for (let i = 0; i < sampleCount; i++) {
      const s = view.getInt16(dataOffset + i * 2, true);
      out[i] = s / 0x8000;
    }
    return out;
  }
  if (bitsPerSample === 32) {
    const sampleCount = dataLength / 4;
    const out = new Float32Array(sampleCount);
    for (let i = 0; i < sampleCount; i++) {
      out[i] = view.getFloat32(dataOffset + i * 4, true);
    }
    return out;
  }
  throw new Error(`bitsPerSample non supporté : ${bitsPerSample}`);
}
