import type { TimerSettings } from "../api";

type SettingsPanelProps = {
  settings: TimerSettings;
  onChange: (settings: TimerSettings) => void;
  onClose: () => void;
};

const PAUSE_OPTIONS = ["0", "1", "2", "3", "5", "unlimited"] as const;

function Toggle({
  label,
  hint,
  checked,
  onChange,
}: {
  label: string;
  hint: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex min-h-11 cursor-pointer items-center justify-between gap-4 py-2">
      <span>
        <span className="block text-[13px] font-medium">{label}</span>
        <span className="block text-xs text-text-muted">{hint}</span>
      </span>
      <input
        type="checkbox"
        role="switch"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="peer sr-only"
      />
      <span
        aria-hidden="true"
        className="relative h-[26px] w-11 shrink-0 rounded-full bg-white/14 transition-colors peer-checked:bg-shield peer-focus-visible:outline-2 peer-focus-visible:outline-focus after:absolute after:top-[3px] after:left-[3px] after:size-5 after:rounded-full after:bg-white after:transition-transform peer-checked:after:translate-x-[18px]"
      />
    </label>
  );
}

function Select({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (value: string) => void;
}) {
  return (
    <label className="flex min-h-11 items-center justify-between gap-4 py-2">
      <span className="text-[13px] font-medium">{label}</span>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="h-9 rounded-control border border-white/10 bg-control px-2.5 text-[13px] text-text"
      >
        {options.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
    </label>
  );
}

export function SettingsPanel({ settings: s, onChange, onClose }: SettingsPanelProps) {
  const update = (patch: Partial<TimerSettings>) => onChange({ ...s, ...patch });
  return (
    <div
      role="dialog"
      aria-label="Timer settings"
      onKeyDown={(e) => {
        if (e.key === "Escape") onClose();
      }}
      className="absolute top-full right-0 z-20 mt-2 w-[340px] rounded-popover border border-white/10 bg-glass-popover p-4 shadow-popover backdrop-blur-[40px]"
    >
      <div className="mb-1 flex items-center justify-between">
        <h2 className="text-sm font-semibold">Timer settings</h2>
        <button
          type="button"
          onClick={onClose}
          className="h-9 rounded-control px-3 text-[13px] text-text-secondary hover:text-text"
        >
          Done
        </button>
      </div>
      <div className="divide-y divide-hairline">
        <Toggle
          label="Overflow"
          hint="Keep counting past the goal"
          checked={s.overflow}
          onChange={(overflow) => update({ overflow })}
        />
        <Toggle
          label="Auto-start breaks"
          hint="Start the break when a session completes"
          checked={s.autoStartBreaks}
          onChange={(autoStartBreaks) => update({ autoStartBreaks })}
        />
        <Toggle
          label="Auto-start next session"
          hint="Start focusing again when a break ends"
          checked={s.autoStartFocus}
          onChange={(autoStartFocus) => update({ autoStartFocus })}
        />
        <Select
          label="Pauses per session"
          value={s.pauseLimit === null ? "unlimited" : String(s.pauseLimit)}
          options={PAUSE_OPTIONS.map((v) => ({
            value: v,
            label: v === "unlimited" ? "Unlimited" : v,
          }))}
          onChange={(v) => update({ pauseLimit: v === "unlimited" ? null : Number(v) })}
        />
        <Select
          label="Long break every"
          value={String(s.longBreakEvery)}
          options={[2, 3, 4, 5, 6, 7, 8].map((n) => ({
            value: String(n),
            label: `${n} sessions`,
          }))}
          onChange={(v) => update({ longBreakEvery: Number(v) })}
        />
      </div>
    </div>
  );
}
