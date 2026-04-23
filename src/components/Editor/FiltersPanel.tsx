import { useEditor } from '../../store/editorStore';
import { PRESETS, applyPreset, filterToCss } from '../../lib/filters';

export function FiltersPanel() {
  const filter = useEditor((s) => s.filter);
  const media = useEditor((s) => s.media);
  const thumbSrc = media?.kind === 'image' ? media.src : undefined;
  const patch = useEditor((s) => s.patchFilter);
  const setFilter = useEditor((s) => s.setFilter);
  const reset = useEditor((s) => s.resetFilter);

  return (
    <div className="space-y-4">
      <div>
        <div className="mb-2 flex items-center justify-between">
          <span className="label mb-0">Presets</span>
          <button className="text-xs text-neutral-400 hover:text-neutral-200" onClick={reset}>
            Réinitialiser
          </button>
        </div>
        <div className="grid grid-cols-4 gap-2">
          {PRESETS.map((p) => {
            const css = filterToCss(applyPreset(filter, p.id));
            const active = filter.preset === p.id;
            return (
              <button
                key={p.id}
                onClick={() => setFilter(applyPreset(filter, p.id))}
                className={`group overflow-hidden rounded-lg border text-[10px] transition ${
                  active ? 'border-brand-500' : 'border-white/10 hover:border-white/30'
                }`}
                title={p.label}
              >
                <div
                  className="aspect-square w-full bg-gradient-to-br from-neutral-700 to-neutral-900"
                  style={{
                    filter: css,
                    backgroundImage: thumbSrc ? `url(${thumbSrc})` : undefined,
                    backgroundSize: 'cover',
                    backgroundPosition: 'center',
                  }}
                />
                <div className="py-1 text-center text-neutral-300">{p.label}</div>
              </button>
            );
          })}
        </div>
      </div>

      <Slider
        label="Luminosité"
        value={filter.brightness}
        min={-0.5}
        max={0.5}
        step={0.01}
        format={(v) => (v * 100).toFixed(0)}
        onChange={(v) => patch({ brightness: v })}
      />
      <Slider
        label="Contraste"
        value={filter.contrast}
        min={-50}
        max={50}
        step={1}
        format={(v) => v.toFixed(0)}
        onChange={(v) => patch({ contrast: v })}
      />
      <Slider
        label="Saturation"
        value={filter.saturation}
        min={-1}
        max={1.5}
        step={0.05}
        format={(v) => (v * 100).toFixed(0)}
        onChange={(v) => patch({ saturation: v })}
      />
      <Slider
        label="Teinte"
        value={filter.hue}
        min={-180}
        max={180}
        step={1}
        format={(v) => `${v.toFixed(0)}°`}
        onChange={(v) => patch({ hue: v })}
      />
      <Slider
        label="Flou"
        value={filter.blur}
        min={0}
        max={20}
        step={0.5}
        format={(v) => v.toFixed(1)}
        onChange={(v) => patch({ blur: v })}
      />

      <div className="flex flex-wrap gap-2">
        <Toggle
          label="N&B"
          value={filter.grayscale}
          onChange={(v) => patch({ grayscale: v })}
        />
        <Toggle label="Sépia" value={filter.sepia} onChange={(v) => patch({ sepia: v })} />
        <Toggle
          label="Inverser"
          value={filter.invert}
          onChange={(v) => patch({ invert: v })}
        />
      </div>
    </div>
  );
}

interface SliderProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  format: (v: number) => string;
  onChange: (v: number) => void;
}

function Slider({ label, value, min, max, step, format, onChange }: SliderProps) {
  return (
    <div>
      <div className="mb-1 flex items-center justify-between">
        <span className="label mb-0">{label}</span>
        <span className="text-xs text-neutral-400">{format(value)}</span>
      </div>
      <input
        type="range"
        className="w-full"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </div>
  );
}

interface ToggleProps {
  label: string;
  value: boolean;
  onChange: (v: boolean) => void;
}

function Toggle({ label, value, onChange }: ToggleProps) {
  return (
    <button
      className={`btn-outline text-xs ${value ? 'ring-1 ring-brand-500' : ''}`}
      onClick={() => onChange(!value)}
    >
      {label}
    </button>
  );
}
