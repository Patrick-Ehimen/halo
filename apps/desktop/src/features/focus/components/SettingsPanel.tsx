import type { ReactNode } from "react";
import { Dropdown } from "@halo/ui";
import type { Prefs } from "../../app/api";
import type { TimerSettings } from "../api";

type SettingsPanelProps = {
  settings: TimerSettings;
  onChange: (settings: TimerSettings) => void;
  prefs: Prefs | null;
  onPrefsChange: (prefs: Prefs) => void;
  onClose: () => void;
};

const CHECK_IN_OPTIONS = ["off", "10", "15", "20", "30"] as const;

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
        className="relative h-[26px] w-11 shrink-0 rounded-full bg-white/14 transition-colors peer-checked:bg-focus peer-focus-visible:outline-2 peer-focus-visible:outline-focus after:absolute after:top-[3px] after:left-[3px] after:size-5 after:rounded-full after:bg-white after:transition-transform peer-checked:after:translate-x-[18px]"
      />
    </label>
  );
}

function Row({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex min-h-11 items-center justify-between gap-4 py-2">
      <span className="text-[13px] font-medium">{label}</span>
      {children}
    </div>
  );
}

export function SettingsPanel({
  settings: s,
  onChange,
  prefs,
  onPrefsChange,
  onClose,
}: SettingsPanelProps) {
  const update = (patch: Partial<TimerSettings>) => onChange({ ...s, ...patch });
  const updatePrefs = (patch: Partial<Prefs>) => prefs && onPrefsChange({ ...prefs, ...patch });
  return (
    <div
      role="dialog"
      aria-label="Timer settings"
      className="absolute top-full right-0 z-20 mt-2 w-[360px] rounded-popover border border-white/10 bg-glass-popover p-4 shadow-popover backdrop-blur-[40px]"
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
        <Row label="Pauses per session">
          <Dropdown
            label="Pauses per session"
            value={s.pauseLimit === null ? "unlimited" : String(s.pauseLimit)}
            options={PAUSE_OPTIONS.map((v) => ({
              value: v,
              label: v === "unlimited" ? "Unlimited" : v,
            }))}
            onChange={(v) => update({ pauseLimit: v === "unlimited" ? null : Number(v) })}
          />
        </Row>
        <Row label="Long break every">
          <Dropdown
            label="Long break every"
            value={String(s.longBreakEvery)}
            options={[2, 3, 4, 5, 6, 7, 8].map((n) => ({
              value: String(n),
              label: `${n} sessions`,
            }))}
            onChange={(v) => update({ longBreakEvery: Number(v) })}
          />
        </Row>
        <Row label="Check in every">
          <Dropdown
            label="Check in every"
            value={s.checkInMinutes === null ? "off" : String(s.checkInMinutes)}
            options={CHECK_IN_OPTIONS.map((v) => ({
              value: v,
              label: v === "off" ? "Off" : `${v} min`,
            }))}
            onChange={(v) => update({ checkInMinutes: v === "off" ? null : Number(v) })}
          />
        </Row>
      </div>

      {prefs && (
        <>
          <h3 className="mt-4 mb-1 text-[11px] font-semibold tracking-[0.06em] text-text-muted uppercase">
            Menu bar and alerts
          </h3>
          <div className="divide-y divide-hairline">
            <Row label="Menu bar shows">
              <div
                role="group"
                aria-label="Menu bar shows"
                className="flex gap-0.5 rounded-control border border-white/7 bg-control p-[3px]"
              >
                {(
                  [
                    ["timer", "Timer"],
                    ["icon", "Icon only"],
                  ] as const
                ).map(([value, label]) => (
                  <button
                    key={value}
                    type="button"
                    aria-pressed={prefs.menubarShows === value}
                    onClick={() => updatePrefs({ menubarShows: value })}
                    className={`h-9 rounded-lg px-3 text-[13px] font-medium ${
                      prefs.menubarShows === value
                        ? "bg-control-selected text-text"
                        : "text-text-secondary hover:text-text"
                    }`}
                  >
                    {label}
                  </button>
                ))}
              </div>
            </Row>
            <Toggle
              label="Show seconds"
              hint="Off keeps the menu bar calmer"
              checked={prefs.menubarSeconds}
              onChange={(menubarSeconds) => updatePrefs({ menubarSeconds })}
            />
            <Toggle
              label="Monochrome icon"
              hint="Matches other menu bar icons"
              checked={prefs.menubarMonochrome}
              onChange={(menubarMonochrome) => updatePrefs({ menubarMonochrome })}
            />
            <Toggle
              label="Notifications"
              hint="When a session completes or a break ends"
              checked={prefs.notifications}
              onChange={(notifications) => updatePrefs({ notifications })}
            />
            <Toggle
              label="Break overlay"
              hint="Cover every screen during breaks"
              checked={prefs.breakOverlay}
              onChange={(breakOverlay) => updatePrefs({ breakOverlay })}
            />
          </div>
        </>
      )}
    </div>
  );
}
