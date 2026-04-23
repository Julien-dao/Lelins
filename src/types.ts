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
  brightness: number;
  contrast: number;
  saturation: number;
  hue: number;
  blur: number;
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
  /** Optional: Instagram/Facebook Business Account or Page ID */
  externalId?: string;
  /** Optional: URL where the media will be temporarily hosted (IG/FB need a public URL) */
  mediaHost?: string;
  createdAt: number;
}

export interface CaptionState {
  text: string;
  hashtags: string[];
  mentions: string[];
}

export type MediaKind = 'image' | 'video';

export interface ImageSource {
  kind: 'image';
  src: string;
  naturalWidth: number;
  naturalHeight: number;
  name: string;
}

export interface VideoSource {
  kind: 'video';
  src: string;
  file: File;
  naturalWidth: number;
  naturalHeight: number;
  duration: number;
  name: string;
}

export type MediaSource = ImageSource | VideoSource;

export interface VideoState {
  currentTime: number;
  trimStart: number;
  trimEnd: number;
  playing: boolean;
}
