import { useEditor } from '../../store/editorStore';

export function LayersPanel() {
  const layers = useEditor((s) => s.layers);
  const selectedId = useEditor((s) => s.selectedLayerId);
  const selectLayer = useEditor((s) => s.selectLayer);
  const addTextLayer = useEditor((s) => s.addTextLayer);
  const removeLayer = useEditor((s) => s.removeLayer);
  const reorderLayer = useEditor((s) => s.reorderLayer);

  return (
    <div className="space-y-3">
      <div className="flex gap-2">
        <button className="btn-outline flex-1" onClick={() => addTextLayer('text')}>
          + Texte
        </button>
        <button className="btn-outline flex-1" onClick={() => addTextLayer('subtitle')}>
          + Sous-titre
        </button>
      </div>
      <div className="space-y-1">
        {layers.length === 0 && (
          <p className="rounded-lg border border-dashed border-white/10 px-3 py-6 text-center text-xs text-neutral-500">
            Aucun calque. Ajoutez du texte ou un sous-titre.
          </p>
        )}
        {[...layers].reverse().map((layer) => {
          const active = layer.id === selectedId;
          return (
            <div
              key={layer.id}
              className={`group flex items-center gap-2 rounded-lg border px-2 py-1.5 text-sm ${
                active ? 'border-brand-500/60 bg-brand-500/10' : 'border-white/5 bg-white/5 hover:bg-white/10'
              }`}
            >
              <button
                className="flex-1 truncate text-left"
                onClick={() => selectLayer(layer.id)}
                title={layer.text}
              >
                <span className="mr-2 inline-block rounded bg-white/10 px-1.5 py-0.5 text-[10px] uppercase text-neutral-300">
                  {layer.kind === 'subtitle' ? 'ST' : 'Txt'}
                </span>
                {layer.text || '(vide)'}
              </button>
              <div className="flex shrink-0 gap-1 opacity-0 transition-opacity group-hover:opacity-100">
                <button
                  className="rounded px-1 text-xs text-neutral-300 hover:bg-white/10"
                  onClick={() => reorderLayer(layer.id, 'up')}
                  title="Monter"
                >
                  ↑
                </button>
                <button
                  className="rounded px-1 text-xs text-neutral-300 hover:bg-white/10"
                  onClick={() => reorderLayer(layer.id, 'down')}
                  title="Descendre"
                >
                  ↓
                </button>
                <button
                  className="rounded px-1 text-xs text-red-300 hover:bg-red-500/20"
                  onClick={() => removeLayer(layer.id)}
                  title="Supprimer"
                >
                  ✕
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
