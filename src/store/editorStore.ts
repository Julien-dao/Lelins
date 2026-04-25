import { create } from 'zustand';
import type {
  FilterState,
  MediaSource,
  SocialAccount,
  SocialNetwork,
  SubtitleSegment,
  SubtitleStyle,
  TextLayer,
  VideoState,
} from '../types';
import { DEFAULT_FILTER } from '../lib/filters';
import { uid } from '../lib/id';
import { FORMATS } from '../lib/formats';

interface EditorState {
  media: MediaSource | null;
  video: VideoState;
  format: SocialNetwork;
  layers: TextLayer[];
  selectedLayerId: string | null;
  filter: FilterState;
  caption: string;
  accounts: SocialAccount[];
  selectedAccountIds: string[];
  subtitleTrack: SubtitleSegment[];
  subtitleStyle: SubtitleStyle;

  setMedia: (media: MediaSource | null) => void;
  setFormat: (format: SocialNetwork) => void;

  setVideoTime: (t: number) => void;
  setVideoTrim: (start: number, end: number) => void;
  setVideoPlaying: (playing: boolean) => void;

  setSubtitleTrack: (segments: SubtitleSegment[]) => void;
  updateSubtitleSegment: (id: string, patch: Partial<SubtitleSegment>) => void;
  removeSubtitleSegment: (id: string) => void;
  clearSubtitleTrack: () => void;
  setSubtitleStyle: (patch: Partial<SubtitleStyle>) => void;

  addTextLayer: (kind: 'text' | 'subtitle') => void;
  updateLayer: (id: string, patch: Partial<TextLayer>) => void;
  removeLayer: (id: string) => void;
  selectLayer: (id: string | null) => void;
  reorderLayer: (id: string, direction: 'up' | 'down') => void;

  setFilter: (filter: FilterState) => void;
  patchFilter: (patch: Partial<FilterState>) => void;
  resetFilter: () => void;

  setCaption: (caption: string) => void;

  addAccount: (account: Omit<SocialAccount, 'id' | 'createdAt'>) => void;
  updateAccount: (id: string, patch: Partial<SocialAccount>) => void;
  removeAccount: (id: string) => void;
  toggleAccount: (id: string) => void;
}

const ACCOUNTS_KEY = 'lelins.accounts.v1';

function loadAccounts(): SocialAccount[] {
  try {
    const raw = localStorage.getItem(ACCOUNTS_KEY);
    if (!raw) return [];
    return JSON.parse(raw) as SocialAccount[];
  } catch {
    return [];
  }
}

function persistAccounts(accounts: SocialAccount[]) {
  try {
    localStorage.setItem(ACCOUNTS_KEY, JSON.stringify(accounts));
  } catch {
    /* quota exceeded or disabled, silently ignore */
  }
}

function makeTextLayer(kind: 'text' | 'subtitle', format: SocialNetwork): TextLayer {
  const spec = FORMATS[format];
  const isSub = kind === 'subtitle';
  return {
    id: uid('layer'),
    kind,
    text: isSub ? 'Votre sous-titre ici' : 'Votre texte',
    x: spec.width * 0.1,
    y: isSub ? spec.height * 0.78 : spec.height * 0.15,
    width: spec.width * 0.8,
    fontSize: isSub ? 56 : 72,
    fontFamily: 'Inter',
    color: '#ffffff',
    stroke: '#000000',
    strokeWidth: isSub ? 2 : 0,
    align: 'center',
    bold: isSub,
    italic: false,
    background: isSub ? 'pill' : 'none',
    backgroundColor: '#000000',
    rotation: 0,
    opacity: 1,
  };
}

const DEFAULT_VIDEO: VideoState = {
  currentTime: 0,
  trimStart: 0,
  trimEnd: 0,
  playing: false,
};

const DEFAULT_SUBTITLE_STYLE: SubtitleStyle = {
  fontFamily: 'Inter',
  fontSize: 56,
  color: '#ffffff',
  background: 'pill',
  backgroundColor: '#000000',
  position: 'bottom',
};

export const useEditor = create<EditorState>((set, get) => ({
  media: null,
  video: DEFAULT_VIDEO,
  format: 'instagram-feed',
  layers: [],
  selectedLayerId: null,
  filter: DEFAULT_FILTER,
  caption: '',
  accounts: loadAccounts(),
  selectedAccountIds: [],
  subtitleTrack: [],
  subtitleStyle: DEFAULT_SUBTITLE_STYLE,

  setMedia: (media) =>
    set({
      media,
      video:
        media?.kind === 'video'
          ? { currentTime: 0, trimStart: 0, trimEnd: media.duration, playing: false }
          : DEFAULT_VIDEO,
      // Importing a new media drops any auto-generated subtitle track.
      subtitleTrack: [],
    }),

  setFormat: (format) => set({ format }),

  setVideoTime: (t) => set((s) => ({ video: { ...s.video, currentTime: t } })),
  setVideoTrim: (start, end) =>
    set((s) => ({
      video: {
        ...s.video,
        trimStart: Math.max(0, start),
        trimEnd: Math.max(start + 0.1, end),
      },
    })),
  setVideoPlaying: (playing) => set((s) => ({ video: { ...s.video, playing } })),

  setSubtitleTrack: (segments) => set({ subtitleTrack: segments }),
  updateSubtitleSegment: (id, patch) =>
    set((s) => ({
      subtitleTrack: s.subtitleTrack.map((seg) =>
        seg.id === id ? { ...seg, ...patch } : seg,
      ),
    })),
  removeSubtitleSegment: (id) =>
    set((s) => ({ subtitleTrack: s.subtitleTrack.filter((seg) => seg.id !== id) })),
  clearSubtitleTrack: () => set({ subtitleTrack: [] }),
  setSubtitleStyle: (patch) =>
    set((s) => ({ subtitleStyle: { ...s.subtitleStyle, ...patch } })),

  addTextLayer: (kind) => {
    const layer = makeTextLayer(kind, get().format);
    set((s) => ({
      layers: [...s.layers, layer],
      selectedLayerId: layer.id,
    }));
  },

  updateLayer: (id, patch) =>
    set((s) => ({
      layers: s.layers.map((l) => (l.id === id ? { ...l, ...patch } : l)),
    })),

  removeLayer: (id) =>
    set((s) => ({
      layers: s.layers.filter((l) => l.id !== id),
      selectedLayerId: s.selectedLayerId === id ? null : s.selectedLayerId,
    })),

  selectLayer: (id) => set({ selectedLayerId: id }),

  reorderLayer: (id, direction) =>
    set((s) => {
      const idx = s.layers.findIndex((l) => l.id === id);
      if (idx === -1) return {};
      const target = direction === 'up' ? idx + 1 : idx - 1;
      if (target < 0 || target >= s.layers.length) return {};
      const layers = [...s.layers];
      [layers[idx], layers[target]] = [layers[target], layers[idx]];
      return { layers };
    }),

  setFilter: (filter) => set({ filter }),
  patchFilter: (patch) => set((s) => ({ filter: { ...s.filter, ...patch, preset: 'custom' } })),
  resetFilter: () => set({ filter: DEFAULT_FILTER }),

  setCaption: (caption) => set({ caption }),

  addAccount: (account) => {
    const full: SocialAccount = { ...account, id: uid('acc'), createdAt: Date.now() };
    set((s) => {
      const accounts = [...s.accounts, full];
      persistAccounts(accounts);
      return { accounts };
    });
  },

  updateAccount: (id, patch) =>
    set((s) => {
      const accounts = s.accounts.map((a) => (a.id === id ? { ...a, ...patch } : a));
      persistAccounts(accounts);
      return { accounts };
    }),

  removeAccount: (id) =>
    set((s) => {
      const accounts = s.accounts.filter((a) => a.id !== id);
      persistAccounts(accounts);
      return {
        accounts,
        selectedAccountIds: s.selectedAccountIds.filter((x) => x !== id),
      };
    }),

  toggleAccount: (id) =>
    set((s) => ({
      selectedAccountIds: s.selectedAccountIds.includes(id)
        ? s.selectedAccountIds.filter((x) => x !== id)
        : [...s.selectedAccountIds, id],
    })),
}));
