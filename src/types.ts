export type SocialNetwork =
  | 'instagram-feed'
  | 'instagram-story'
  | 'instagram-reel'
  | 'tiktok'
  | 'youtube-shorts'
  | 'x'
  | 'linkedin'
  | 'facebook';

export interface FormatSpec {
  id: SocialNetwork;
  label: string;
  short: string;
  width: number;
  height: number;
  aspect: string;
  captionMax: number;
  hashtagMax: number;
}

export interface TextLayer {
  id: string;
  kind: 'text' | 'subtitle';
  text: string;
  x: number;
  y: number;
  width: number;
  fontSize: number;
  fontFamily: string;
  color: string;
  stroke: string;
  strokeWidth: number;
  align: 'left' | 'center' | 'right';
  bold: boolean;
  italic: boolean;
  background: 'none' | 'solid' | 'pill';
  backgroundColor: string;
  rotation: number;
  opacity: number;
}

export interface FilterState {
  brightness: number; // -1 .. 1 (0 = neutral)
  contrast: number;   // -100 .. 100 (0 = neutral, Konva convention)
  saturation: number; // -2 .. 10 (0 = neutral, Konva hsl)
  hue: number;        // -180 .. 180
  blur: number;       // 0 .. 20
  grayscale: boolean;
  sepia: boolean;
  invert: boolean;
  preset: string;
}

export interface SocialAccount {
  id: string;
  network: SocialNetwork;
  handle: string;
  token: string;
  createdAt: number;
}

export interface CaptionState {
  text: string;
  hashtags: string[];
  mentions: string[];
}
