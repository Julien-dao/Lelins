import type { FilterState } from '../types';

export const DEFAULT_FILTER: FilterState = {
  brightness: 0,
  contrast: 0,
  saturation: 0,
  hue: 0,
  blur: 0,
  grayscale: false,
  sepia: false,
  invert: false,
  preset: 'none',
};

export interface FilterPreset {
  id: string;
  label: string;
  values: Partial<FilterState>;
}

export const PRESETS: FilterPreset[] = [
  { id: 'none', label: 'Original', values: {} },
  {
    id: 'warm',
    label: 'Chaud',
    values: { brightness: 0.05, contrast: 10, saturation: 0.3, hue: 10 },
  },
  {
    id: 'cool',
    label: 'Froid',
    values: { brightness: 0.02, contrast: 8, saturation: 0.2, hue: -15 },
  },
  {
    id: 'vintage',
    label: 'Vintage',
    values: { brightness: -0.05, contrast: -10, saturation: -0.4, sepia: true },
  },
  {
    id: 'mono',
    label: 'Mono',
    values: { contrast: 15, grayscale: true },
  },
  {
    id: 'punchy',
    label: 'Punchy',
    values: { brightness: 0.05, contrast: 25, saturation: 0.6 },
  },
  {
    id: 'fade',
    label: 'Fade',
    values: { brightness: 0.08, contrast: -20, saturation: -0.2 },
  },
  {
    id: 'noir',
    label: 'Noir',
    values: { contrast: 40, grayscale: true, brightness: -0.05 },
  },
];

export function applyPreset(current: FilterState, presetId: string): FilterState {
  const preset = PRESETS.find((p) => p.id === presetId);
  if (!preset) return current;
  return {
    ...DEFAULT_FILTER,
    ...preset.values,
    preset: presetId,
  };
}

/**
 * CSS filter string used to render live preview in the DOM (thumbnails, small previews).
 * Konva uses its own filter pipeline for the main canvas.
 */
export function filterToCss(f: FilterState): string {
  const parts: string[] = [];
  parts.push(`brightness(${1 + f.brightness})`);
  parts.push(`contrast(${1 + f.contrast / 100})`);
  parts.push(`saturate(${1 + f.saturation / 2})`);
  if (f.hue !== 0) parts.push(`hue-rotate(${f.hue}deg)`);
  if (f.blur > 0) parts.push(`blur(${f.blur / 4}px)`);
  if (f.grayscale) parts.push('grayscale(1)');
  if (f.sepia) parts.push('sepia(0.6)');
  if (f.invert) parts.push('invert(1)');
  return parts.join(' ');
}
