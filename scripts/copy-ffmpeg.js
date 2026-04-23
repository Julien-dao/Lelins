#!/usr/bin/env node
/**
 * Copie les binaires @ffmpeg/core (mode single-thread, sans SharedArrayBuffer)
 * vers public/ffmpeg/ pour que l'app serve FFmpeg entièrement en local.
 */
import { cpSync, existsSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');
const src = resolve(root, 'node_modules/@ffmpeg/core/dist/umd');
const dst = resolve(root, 'public/ffmpeg');

if (!existsSync(src)) {
  console.warn('[copy-ffmpeg] @ffmpeg/core introuvable, saut.');
  process.exit(0);
}

mkdirSync(dst, { recursive: true });
for (const file of ['ffmpeg-core.js', 'ffmpeg-core.wasm']) {
  cpSync(resolve(src, file), resolve(dst, file));
  console.log(`[copy-ffmpeg] ${file} → public/ffmpeg/`);
}
