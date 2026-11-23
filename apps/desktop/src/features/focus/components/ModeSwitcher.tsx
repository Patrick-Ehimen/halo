import type { PresetId } from "../api";
import { PRESETS } from "../format";

type ModeSwitcherProps = {
  value: PresetId;
  disabled: boolean;
  onChange: (preset: PresetId) => void;
};

export function ModeSwitcher({ value, disabled, onChange }: ModeSwitcherProps) {
  return (
    <div
      role="group"
      aria-label="Timer mode"
      className="flex gap-0.5 rounded-control border border-white/7 bg-control p-[3px]"
    >
      {PRESETS.map((p) => {
        const on = p.id === value;
        return (
          <button
            key={p.id}
            type="button"
            aria-pressed={on}
            disabled={disabled && !on}
            onClick={() => onChange(p.id)}
            className={`h-9 rounded-lg px-3.5 text-[13px] font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-40 ${
              on ? "bg-control-selected text-focus" : "text-text-secondary hover:text-text"
            }`}
          >
            {p.label}
          </button>
        );
      })}
    </div>
  );
}
