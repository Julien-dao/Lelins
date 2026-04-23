import { useEditor } from '../../store/editorStore';

const FONTS = [
  'Inter',
  'Arial',
  'Helvetica',
  'Georgia',
  'Times New Roman',
  'Courier New',
  'Impact',
  'Verdana',
  'Trebuchet MS',
];

export function TextInspector() {
  const selectedId = useEditor((s) => s.selectedLayerId);
  const layer = useEditor((s) => s.layers.find((l) => l.id === selectedId));
  const update = useEditor((s) => s.updateLayer);

  if (!layer) {
    return (
      <p className="rounded-lg border border-dashed border-white/10 px-3 py-6 text-center text-xs text-neutral-500">
        Sélectionnez un calque pour éditer son texte.
      </p>
    );
  }

  return (
    <div className="space-y-3">
      <div>
        <label className="label">Contenu</label>
        <textarea
          className="input min-h-[72px]"
          value={layer.text}
          onChange={(e) => update(layer.id, { text: e.target.value })}
        />
      </div>

      <div className="grid grid-cols-2 gap-2">
        <div>
          <label className="label">Police</label>
          <select
            className="input"
            value={layer.fontFamily}
            onChange={(e) => update(layer.id, { fontFamily: e.target.value })}
          >
            {FONTS.map((f) => (
              <option key={f} value={f}>
                {f}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label className="label">Taille</label>
          <input
            type="number"
            className="input"
            value={layer.fontSize}
            min={8}
            max={400}
            onChange={(e) => update(layer.id, { fontSize: Number(e.target.value) })}
          />
        </div>
      </div>

      <div className="flex gap-2">
        <button
          className={`btn-outline flex-1 ${layer.bold ? 'ring-1 ring-brand-500' : ''}`}
          onClick={() => update(layer.id, { bold: !layer.bold })}
        >
          <span className="font-bold">B</span>
        </button>
        <button
          className={`btn-outline flex-1 ${layer.italic ? 'ring-1 ring-brand-500' : ''}`}
          onClick={() => update(layer.id, { italic: !layer.italic })}
        >
          <span className="italic">I</span>
        </button>
        <select
          className="input flex-1"
          value={layer.align}
          onChange={(e) => update(layer.id, { align: e.target.value as 'left' | 'center' | 'right' })}
        >
          <option value="left">Gauche</option>
          <option value="center">Centré</option>
          <option value="right">Droite</option>
        </select>
      </div>

      <div className="grid grid-cols-2 gap-2">
        <div>
          <label className="label">Couleur</label>
          <input
            type="color"
            className="input h-10 cursor-pointer p-1"
            value={layer.color}
            onChange={(e) => update(layer.id, { color: e.target.value })}
          />
        </div>
        <div>
          <label className="label">Contour</label>
          <input
            type="color"
            className="input h-10 cursor-pointer p-1"
            value={layer.stroke}
            onChange={(e) => update(layer.id, { stroke: e.target.value })}
          />
        </div>
      </div>

      <div>
        <label className="label">Épaisseur contour · {layer.strokeWidth}</label>
        <input
          type="range"
          min={0}
          max={12}
          step={0.5}
          value={layer.strokeWidth}
          onChange={(e) => update(layer.id, { strokeWidth: Number(e.target.value) })}
          className="w-full"
        />
      </div>

      <div>
        <label className="label">Fond</label>
        <div className="flex gap-2">
          {(['none', 'solid', 'pill'] as const).map((v) => (
            <button
              key={v}
              className={`btn-outline flex-1 text-xs ${layer.background === v ? 'ring-1 ring-brand-500' : ''}`}
              onClick={() => update(layer.id, { background: v })}
            >
              {v === 'none' ? 'Aucun' : v === 'solid' ? 'Bloc' : 'Pilule'}
            </button>
          ))}
        </div>
      </div>
      {layer.background !== 'none' && (
        <div>
          <label className="label">Couleur du fond</label>
          <input
            type="color"
            className="input h-10 cursor-pointer p-1"
            value={layer.backgroundColor}
            onChange={(e) => update(layer.id, { backgroundColor: e.target.value })}
          />
        </div>
      )}

      <div>
        <label className="label">Opacité · {Math.round(layer.opacity * 100)}%</label>
        <input
          type="range"
          min={0}
          max={1}
          step={0.05}
          value={layer.opacity}
          onChange={(e) => update(layer.id, { opacity: Number(e.target.value) })}
          className="w-full"
        />
      </div>

      <div>
        <label className="label">Rotation · {Math.round(layer.rotation)}°</label>
        <input
          type="range"
          min={-180}
          max={180}
          step={1}
          value={layer.rotation}
          onChange={(e) => update(layer.id, { rotation: Number(e.target.value) })}
          className="w-full"
        />
      </div>
    </div>
  );
}
