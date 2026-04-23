import type { FormatSpec, SocialNetwork } from '../types';

export const FORMATS: Record<SocialNetwork, FormatSpec> = {
  'instagram-feed': {
    id: 'instagram-feed',
    label: 'Instagram · Feed',
    short: 'IG Feed',
    width: 1080,
    height: 1350,
    aspect: '4:5',
    captionMax: 2200,
    hashtagMax: 30,
  },
  'instagram-story': {
    id: 'instagram-story',
    label: 'Instagram · Story',
    short: 'IG Story',
    width: 1080,
    height: 1920,
    aspect: '9:16',
    captionMax: 2200,
    hashtagMax: 30,
  },
  'instagram-reel': {
    id: 'instagram-reel',
    label: 'Instagram · Reel',
    short: 'IG Reel',
    width: 1080,
    height: 1920,
    aspect: '9:16',
    captionMax: 2200,
    hashtagMax: 30,
  },
  tiktok: {
    id: 'tiktok',
    label: 'TikTok',
    short: 'TikTok',
    width: 1080,
    height: 1920,
    aspect: '9:16',
    captionMax: 2200,
    hashtagMax: 100,
  },
  'youtube-shorts': {
    id: 'youtube-shorts',
    label: 'YouTube Shorts',
    short: 'YT Shorts',
    width: 1080,
    height: 1920,
    aspect: '9:16',
    captionMax: 5000,
    hashtagMax: 15,
  },
  x: {
    id: 'x',
    label: 'X (Twitter)',
    short: 'X',
    width: 1200,
    height: 675,
    aspect: '16:9',
    captionMax: 280,
    hashtagMax: 10,
  },
  linkedin: {
    id: 'linkedin',
    label: 'LinkedIn',
    short: 'LinkedIn',
    width: 1200,
    height: 1200,
    aspect: '1:1',
    captionMax: 3000,
    hashtagMax: 10,
  },
  facebook: {
    id: 'facebook',
    label: 'Facebook',
    short: 'Facebook',
    width: 1200,
    height: 630,
    aspect: '1.91:1',
    captionMax: 63206,
    hashtagMax: 30,
  },
};

export const FORMAT_LIST: FormatSpec[] = Object.values(FORMATS);

export function getFormat(id: SocialNetwork): FormatSpec {
  return FORMATS[id];
}
