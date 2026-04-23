import { useCallback, useEffect, useRef, useState } from 'react';
import { Toolbar } from './components/Editor/Toolbar';
import { EditorCanvas, type EditorCanvasHandle } from './components/Editor/EditorCanvas';
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
import { publish } from './lib/publishers';

export default function App() {
  const canvasRef = useRef<EditorCanvasHandle>(null);
  const [containerRef, size] = useElementSize<HTMLDivElement>();
  const [thumbnail, setThumbnail] = useState<string | null>(null);
  const [publishing, setPublishing] = useState(false);
  const [toast, setToast] = useState<string | null>(null);

  const image = useEditor((s) => s.image);
  const layers = useEditor((s) => s.layers);
  const filter = useEditor((s) => s.filter);
  const caption = useEditor((s) => s.caption);
  const format = useEditor((s) => s.format);
  const accounts = useEditor((s) => s.accounts);
  const selectedAccountIds = useEditor((s) => s.selectedAccountIds);

  // Keep a low-res thumbnail of the canvas in sync for the social previews.
  useEffect(() => {
    if (!image) {
      setThumbnail(null);
      return;
    }
    const id = setTimeout(() => {
      const url = canvasRef.current?.toDataURL('image/jpeg', 0.7);
      if (url) setThumbnail(url);
    }, 150);
    return () => clearTimeout(id);
  }, [image, layers, filter, format]);

  const notify = useCallback((msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 3200);
  }, []);

  const handleExport = useCallback(
    async (type: 'png' | 'jpeg') => {
      if (!image) return;
      const blob = await canvasRef.current?.toBlob(
        type === 'png' ? 'image/png' : 'image/jpeg',
        0.95,
      );
      if (!blob) return;
      const base = image.name.replace(/\.[^.]+$/, '') || 'lelins-export';
      downloadBlob(blob, `${base}-${format}.${type === 'png' ? 'png' : 'jpg'}`);
      notify(`Export ${type.toUpperCase()} téléchargé.`);
    },
    [image, format, notify],
  );

  const handleShare = useCallback(async () => {
    if (!image) return;
    const blob = await canvasRef.current?.toBlob('image/jpeg', 0.9);
    if (!blob) return;
    const file = new File([blob], `${format}.jpg`, { type: 'image/jpeg' });
    if (!canShareFiles([file])) {
      // Fallback: copy caption to clipboard + download file.
      try {
        await navigator.clipboard.writeText(caption);
        notify('Partage natif indisponible — légende copiée, fichier téléchargé.');
      } catch {
        notify('Partage natif indisponible — fichier téléchargé.');
      }
      downloadBlob(blob, file.name);
      return;
    }
    try {
      await shareFiles([file], caption);
      notify('Partage lancé.');
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      if (!msg.toLowerCase().includes('abort')) notify(`Partage annulé : ${msg}`);
    }
  }, [image, format, caption, notify]);

  const handlePublish = useCallback(async () => {
    if (!image || selectedAccountIds.length === 0) return;
    setPublishing(true);
    try {
      const blob = await canvasRef.current?.toBlob('image/jpeg', 0.95);
      if (!blob) {
        notify('Impossible de générer le média.');
        return;
      }
      const file = new File([blob], `${format}.jpg`, { type: 'image/jpeg' });
      const targets = accounts.filter((a) => selectedAccountIds.includes(a.id));
      const results = await Promise.all(
        targets.map((account) => publish({ account, caption, file })),
      );
      const configured = results.filter((r) => r.status === 'queued').length;
      const pending = results.filter((r) => r.status === 'needs-config').length;
      notify(
        configured > 0
          ? `${configured} publication(s) envoyée(s). ${pending} en attente de configuration.`
          : `${pending} compte(s) : implémentation publisher prévue en itération 2.`,
      );
    } finally {
      setPublishing(false);
    }
  }, [image, selectedAccountIds, accounts, caption, format, notify]);

  return (
    <div className="flex h-full flex-col">
      <Toolbar
        onExport={handleExport}
        onShare={handleShare}
        onPublish={handlePublish}
        publishing={publishing}
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
              content: <AccountsPanel />,
            },
          ]}
          initialTab="preview"
        />
      </div>

      {toast && (
        <div className="pointer-events-none fixed bottom-4 left-1/2 z-50 -translate-x-1/2">
          <div className="pointer-events-auto rounded-lg border border-white/10 bg-neutral-900/95 px-4 py-2 text-sm shadow-xl">
            {toast}
          </div>
        </div>
      )}
    </div>
  );
}
