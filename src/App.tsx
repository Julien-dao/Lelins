import { useCallback, useEffect, useRef, useState } from 'react';
import { Toolbar } from './components/Editor/Toolbar';
import { EditorCanvas, type EditorCanvasHandle } from './components/Editor/EditorCanvas';
import { VideoTimeline } from './components/Editor/VideoTimeline';
import { LayersPanel } from './components/Editor/LayersPanel';
import { TextInspector } from './components/Editor/TextInspector';
import { FiltersPanel } from './components/Editor/FiltersPanel';
import { CaptionEditor } from './components/Caption/CaptionEditor';
import { SocialPreviews } from './components/Preview/SocialPreviews';
import { AccountsPanel } from './components/Accounts/AccountsPanel';
import { Sidebar } from './components/Layout/Sidebar';
import { useElementSize } from './hooks/useElementSize';
import { useEditor } from './store/editorStore';
import { canShareFiles, downloadBlob, shareFiles } from './lib/export';
import { publish, type PublishResult } from './lib/publishers';
import { exportVideo } from './lib/videoExport';
import { renderOverlayPng } from './lib/overlayRender';
import { getFormat } from './lib/formats';

export default function App() {
  const canvasRef = useRef<EditorCanvasHandle>(null);
  const [containerRef, size] = useElementSize<HTMLDivElement>();
  const [thumbnail, setThumbnail] = useState<string | null>(null);
  const [publishing, setPublishing] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [exportProgress, setExportProgress] = useState<number | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const [publishResults, setPublishResults] = useState<PublishResult[]>([]);

  const media = useEditor((s) => s.media);
  const layers = useEditor((s) => s.layers);
  const filter = useEditor((s) => s.filter);
  const caption = useEditor((s) => s.caption);
  const format = useEditor((s) => s.format);
  const video = useEditor((s) => s.video);
  const accounts = useEditor((s) => s.accounts);
  const selectedAccountIds = useEditor((s) => s.selectedAccountIds);

  // Keep a low-res thumbnail in sync for the social previews.
  useEffect(() => {
    if (!media) {
      setThumbnail(null);
      return;
    }
    const id = setTimeout(() => {
      const url = canvasRef.current?.toDataURL('image/jpeg', 0.7);
      if (url) setThumbnail(url);
    }, 200);
    return () => clearTimeout(id);
  }, [media, layers, filter, format, video.currentTime]);

  const notify = useCallback((msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 3600);
  }, []);

  const handleExport = useCallback(
    async (type: 'png' | 'jpeg' | 'mp4') => {
      if (!media) return;
      if (type === 'mp4') {
        if (media.kind !== 'video') return;
        setExporting(true);
        setExportProgress(0);
        try {
          const spec = getFormat(format);
          const overlay = await renderOverlayPng(layers, spec.width, spec.height);
          const blob = await exportVideo({
            source: media,
            trimStart: video.trimStart,
            trimEnd: video.trimEnd,
            width: spec.width,
            height: spec.height,
            overlayPng: overlay,
            filter,
            onProgress: (p) => setExportProgress(p),
          });
          const base = media.name.replace(/\.[^.]+$/, '') || 'lelins-export';
          downloadBlob(blob, `${base}-${format}.mp4`);
          notify('Export MP4 téléchargé.');
        } catch (err) {
          const msg = err instanceof Error ? err.message : String(err);
          notify(`Erreur export vidéo : ${msg}`);
        } finally {
          setExporting(false);
          setExportProgress(null);
        }
        return;
      }

      // Image exports.
      const blob = await canvasRef.current?.toBlob(
        type === 'png' ? 'image/png' : 'image/jpeg',
        0.95,
      );
      if (!blob) return;
      const base = media.name.replace(/\.[^.]+$/, '') || 'lelins-export';
      downloadBlob(blob, `${base}-${format}.${type === 'png' ? 'png' : 'jpg'}`);
      notify(`Export ${type.toUpperCase()} téléchargé.`);
    },
    [media, format, layers, video, filter, notify],
  );

  const buildShareFile = useCallback(async (): Promise<File | null> => {
    if (!media) return null;
    if (media.kind === 'video') {
      // Partage du fichier original — le partage natif ne burnera pas les overlays.
      // (Export MP4 pour inclure les overlays.)
      return media.file;
    }
    const blob = await canvasRef.current?.toBlob('image/jpeg', 0.9);
    if (!blob) return null;
    return new File([blob], `${format}.jpg`, { type: 'image/jpeg' });
  }, [media, format]);

  const handleShare = useCallback(async () => {
    const file = await buildShareFile();
    if (!file) return;
    if (!canShareFiles([file])) {
      try {
        await navigator.clipboard.writeText(caption);
        notify('Partage natif indisponible — légende copiée, fichier téléchargé.');
      } catch {
        notify('Partage natif indisponible — fichier téléchargé.');
      }
      downloadBlob(file, file.name);
      return;
    }
    try {
      await shareFiles([file], caption);
      notify('Partage lancé.');
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      if (!msg.toLowerCase().includes('abort')) notify(`Partage annulé : ${msg}`);
    }
  }, [buildShareFile, caption, notify]);

  const handlePublish = useCallback(async () => {
    if (!media || selectedAccountIds.length === 0) return;
    setPublishing(true);
    setPublishResults([]);
    try {
      let file: File | null;
      if (media.kind === 'video') {
        // Pour la publication, burn les overlays via FFmpeg avant envoi.
        setExporting(true);
        setExportProgress(0);
        try {
          const spec = getFormat(format);
          const overlay = await renderOverlayPng(layers, spec.width, spec.height);
          const blob = await exportVideo({
            source: media,
            trimStart: video.trimStart,
            trimEnd: video.trimEnd,
            width: spec.width,
            height: spec.height,
            overlayPng: overlay,
            filter,
            onProgress: (p) => setExportProgress(p),
          });
          file = new File([blob], `${format}.mp4`, { type: 'video/mp4' });
        } finally {
          setExporting(false);
          setExportProgress(null);
        }
      } else {
        const blob = await canvasRef.current?.toBlob('image/jpeg', 0.95);
        file = blob ? new File([blob], `${format}.jpg`, { type: 'image/jpeg' }) : null;
      }
      if (!file) {
        notify('Impossible de générer le média.');
        return;
      }
      const targets = accounts.filter((a) => selectedAccountIds.includes(a.id));
      const results = await Promise.all(
        targets.map((account) => publish({ account, caption, file: file! })),
      );
      setPublishResults(results);
      const ok = results.filter((r) => r.status === 'success').length;
      const fail = results.length - ok;
      notify(
        fail === 0
          ? `${ok} publication(s) réussie(s).`
          : `${ok} OK · ${fail} en échec — voir l'onglet Comptes.`,
      );
    } finally {
      setPublishing(false);
    }
  }, [media, selectedAccountIds, accounts, caption, format, layers, video, filter, notify]);

  return (
    <div className="flex h-full flex-col">
      <Toolbar
        onExport={handleExport}
        onShare={handleShare}
        onPublish={handlePublish}
        publishing={publishing}
        exporting={exporting}
        exportProgress={exportProgress}
      />

      <div className="flex flex-1 overflow-hidden">
        <Sidebar
          side="left"
          tabs={[
            { id: 'layers', label: 'Calques', icon: '📑', content: <LayersPanel /> },
            { id: 'text', label: 'Texte', icon: 'T', content: <TextInspector /> },
            { id: 'filters', label: 'Filtres', icon: '🎨', content: <FiltersPanel /> },
          ]}
        />

        <main ref={containerRef} className="relative flex-1 overflow-hidden bg-neutral-900">
          <EditorCanvas
            ref={canvasRef}
            containerWidth={size.width}
            containerHeight={size.height}
          />
        </main>

        <Sidebar
          side="right"
          tabs={[
            {
              id: 'preview',
              label: 'Aperçus',
              icon: '📱',
              content: <SocialPreviews thumbnailUrl={thumbnail} />,
            },
            {
              id: 'caption',
              label: 'Légende',
              icon: '✏️',
              content: <CaptionEditor />,
            },
            {
              id: 'accounts',
              label: 'Comptes',
              icon: '👥',
              content: (
                <div className="space-y-4">
                  <AccountsPanel />
                  {publishResults.length > 0 && (
                    <div className="rounded-lg border border-white/10 bg-white/5 p-3">
                      <div className="mb-2 text-xs font-semibold text-neutral-300">
                        Dernière publication
                      </div>
                      <ul className="space-y-1 text-[11px]">
                        {publishResults.map((r) => (
                          <li key={r.accountId} className="flex gap-2">
                            <span
                              className={
                                r.status === 'success'
                                  ? 'text-green-400'
                                  : r.status === 'cors-blocked'
                                    ? 'text-amber-400'
                                    : r.status === 'needs-config'
                                      ? 'text-sky-400'
                                      : 'text-red-400'
                              }
                            >
                              ●
                            </span>
                            <span className="flex-1">
                              {r.message}
                              {r.url && (
                                <a
                                  href={r.url}
                                  target="_blank"
                                  rel="noreferrer"
                                  className="ml-1 text-brand-400 underline"
                                >
                                  Ouvrir
                                </a>
                              )}
                            </span>
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
              ),
            },
          ]}
          initialTab="preview"
        />
      </div>

      {media?.kind === 'video' && (
        <VideoTimeline getVideoElement={() => canvasRef.current?.getVideoElement() ?? null} />
      )}

      {toast && (
        <div className="pointer-events-none fixed bottom-16 left-1/2 z-50 -translate-x-1/2">
          <div className="pointer-events-auto rounded-lg border border-white/10 bg-neutral-900/95 px-4 py-2 text-sm shadow-xl">
            {toast}
          </div>
        </div>
      )}
    </div>
  );
}
