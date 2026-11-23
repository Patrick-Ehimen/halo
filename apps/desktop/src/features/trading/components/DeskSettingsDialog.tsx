import { ConfirmDialog, Dropdown, useDismiss } from "@halo/ui";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { tradingApi, type MarketId, type TradingSettings, type TradingView } from "../api";
import { useTradingStore } from "../store";
import { SourceForm } from "./SourceForm";
import { Switch } from "./Switch";

const MARKETS: { id: MarketId; label: string }[] = [
  { id: "sydney", label: "Sydney" },
  { id: "tokyo", label: "Tokyo" },
  { id: "london", label: "London" },
  { id: "newYork", label: "New York" },
];

const DAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/** Common zones; the saved one is always listed. Names only: Rust does the maths. */
const ZONES = [
  "Africa/Lagos",
  "Africa/Johannesburg",
  "Africa/Nairobi",
  "Europe/London",
  "Europe/Berlin",
  "Asia/Dubai",
  "Asia/Kolkata",
  "Asia/Singapore",
  "Asia/Tokyo",
  "Australia/Sydney",
  "America/New_York",
  "America/Chicago",
  "America/Los_Angeles",
  "America/Sao_Paulo",
];

const NUDGE = [3, 5, 10, 15, 30];

const splitList = (s: string) =>
  s
    .split(",")
    .map((x) => x.trim())
    .filter(Boolean);

function Group({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div className="flex flex-col gap-2 border-t border-hairline pt-3.5 first:border-t-0 first:pt-0">
      <h3 className="text-[11px] font-semibold tracking-[0.06em] text-text-muted uppercase">
        {title}
      </h3>
      {children}
    </div>
  );
}

/**
 * Desk settings, in a dialog: your trading window, the zone times show in,
 * the menu bar countdown, and what counts as chart time.
 */
export function DeskSettingsDialog({
  settings,
  view,
  focusSources,
  onClose,
}: {
  settings: TradingSettings;
  view: TradingView;
  /** Opened from chart time: start at the apps and sites. */
  focusSources?: boolean;
  onClose: () => void;
}) {
  const { saveSettings, run, error } = useTradingStore();
  const save = (patch: Partial<TradingSettings>) => void saveSettings({ ...settings, ...patch });
  const zones = ZONES.includes(settings.zone) ? ZONES : [settings.zone, ...ZONES];
  /** The source being edited, and the one waiting for "Remove" to be confirmed. */
  const [editing, setEditing] = useState<number | null>(null);
  const [label, setLabel] = useState("");
  const [site, setSite] = useState("");
  const [removing, setRemoving] = useState<number | null>(null);
  const ref = useRef<HTMLDivElement>(null);
  const sourcesRef = useRef<HTMLDivElement>(null);
  // While a confirmation is up, Escape and outside clicks belong to it.
  useDismiss(ref, onClose, removing === null);
  useEffect(() => {
    if (focusSources) sourcesRef.current?.scrollIntoView?.({ block: "start" });
  }, [focusSources]);

  return (
    <div className="fixed inset-0 z-30 flex items-center justify-center bg-black/50 p-6">
      <div
        ref={ref}
        role="dialog"
        aria-modal="true"
        aria-labelledby="desk-settings-title"
        className="flex max-h-full w-[560px] flex-col rounded-popover border border-white/10 bg-surface shadow-popover"
      >
        <div className="flex items-center justify-between px-5 pt-4 pb-3">
          <h2 id="desk-settings-title" className="text-base font-semibold">
            Desk settings
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="h-11 rounded-control px-3 text-[13px] text-text-secondary hover:text-text"
          >
            Done
          </button>
        </div>
        <div className="flex min-h-0 flex-col gap-3.5 overflow-y-auto px-5 pb-5">
          <Group title="Trading window">
            <div
              role="group"
              aria-label="Trading window market"
              className="flex gap-0.5 self-start rounded-control border border-white/7 bg-white/5 p-[3px]"
            >
              {MARKETS.map((m) => (
                <button
                  key={m.id}
                  type="button"
                  aria-pressed={settings.market === m.id}
                  onClick={() => save({ market: m.id })}
                  className={`h-9 rounded-lg px-3 text-[13px] font-medium ${
                    settings.market === m.id
                      ? "bg-white/12 text-text"
                      : "text-text-secondary hover:text-text"
                  }`}
                >
                  {m.label}
                </button>
              ))}
            </div>
            <div role="group" aria-label="Trading days" className="flex gap-1">
              {DAYS.map((d, i) => {
                const on = settings.days[i] ?? false;
                return (
                  <button
                    key={d}
                    type="button"
                    aria-pressed={on}
                    onClick={() => save({ days: settings.days.map((x, j) => (j === i ? !x : x)) })}
                    className={`h-11 w-11 rounded-lg border text-xs font-medium ${
                      on
                        ? "border-focus/45 bg-focus/14 text-[#FFE2A6]"
                        : "border-white/9 bg-white/4 text-text-secondary"
                    }`}
                  >
                    {d}
                  </button>
                );
              })}
            </div>
            <Switch
              label="Trading countdown in the menu bar"
              hint="When no session runs, show the time to your window"
              checked={settings.menubarCountdown}
              onChange={(menubarCountdown) => save({ menubarCountdown })}
            />
          </Group>

          <Group title="Time zone">
            <div className="flex items-center justify-between gap-4">
              <span className="text-[13px] font-medium">Show times in</span>
              <Dropdown
                label="Time zone"
                value={settings.zone}
                options={zones.map((z) => ({ value: z, label: z.replace("_", " ") }))}
                onChange={(zone) => save({ zone })}
              />
            </div>
          </Group>

          <div ref={sourcesRef} className="flex flex-col gap-3.5">
            <Group title="Chart time">
              <div className="flex items-center justify-between gap-4">
                <span className="text-[13px] font-medium">Nudge after</span>
                <Dropdown
                  label="Nudge after"
                  value={String(settings.nudgeMinutes)}
                  options={NUDGE.map((n) => ({ value: String(n), label: `${n} min` }))}
                  onChange={(v) => save({ nudgeMinutes: Number(v) })}
                />
              </div>
              <Switch
                label="Count charting sites in browsers"
                hint="Asks macOS for Automation access to Safari, Chrome, Arc and others. Only the site name is kept, on this Mac."
                checked={settings.trackBrowsers}
                onChange={(trackBrowsers) => save({ trackBrowsers })}
              />
              {view.chart.browserNote && (
                <div className="flex flex-col gap-2 rounded-xl border border-dashed border-white/16 bg-white/4 px-3.5 py-3">
                  <p className="text-xs leading-normal text-text-secondary">
                    {view.chart.browserNote}
                  </p>
                  {view.chart.browserBlocked && (
                    <button
                      type="button"
                      onClick={() => void run(tradingApi.openAutomationSettings)}
                      className="h-11 self-start rounded-[9px] bg-white/10 px-3 text-xs font-medium text-text"
                    >
                      Open Automation settings
                    </button>
                  )}
                </div>
              )}
            </Group>

            <Group title="Charting apps and sites">
              {settings.sources.length === 0 && (
                <p className="text-[13px] text-text-muted">
                  Nothing tracked yet. Add an app or site below.
                </p>
              )}
              {settings.sources.map((s, i) =>
                editing === i ? (
                  <SourceForm
                    key={`edit-${i}`}
                    initial={s}
                    submitLabel="Save"
                    onCancel={() => setEditing(null)}
                    onSubmit={(next) => {
                      save({ sources: settings.sources.map((x, j) => (j === i ? next : x)) });
                      setEditing(null);
                    }}
                  />
                ) : (
                  <div key={`${s.label}-${i}`} className="flex items-center gap-2 text-[13px]">
                    <span className="w-28 shrink-0 truncate font-medium">{s.label}</span>
                    <span className="min-w-0 flex-1 truncate text-text-secondary">
                      {[...s.apps.map((a) => `${a} app`), ...s.hosts].join(", ")}
                    </span>
                    <button
                      type="button"
                      aria-label={`Edit ${s.label}`}
                      onClick={() => setEditing(i)}
                      className="h-11 rounded-lg px-2.5 text-xs font-medium text-text-secondary hover:bg-white/5 hover:text-text"
                    >
                      Edit
                    </button>
                    <button
                      type="button"
                      aria-label={`Stop tracking ${s.label}`}
                      onClick={() => setRemoving(i)}
                      className="size-11 rounded-lg text-text-muted hover:bg-white/5 hover:text-text"
                    >
                      ×
                    </button>
                  </div>
                ),
              )}
              <form
                className="flex gap-2"
                onSubmit={(e) => {
                  e.preventDefault();
                  if (!label.trim() || !site.trim()) return;
                  save({
                    sources: [
                      ...settings.sources,
                      // Rust sorts these: "bybit.com" is a site, "Bybit" an app.
                      { label: label.trim(), apps: [], hosts: splitList(site) },
                    ],
                  });
                  setLabel("");
                  setSite("");
                }}
              >
                <label htmlFor="source-label" className="sr-only">
                  Name
                </label>
                <input
                  id="source-label"
                  value={label}
                  onChange={(e) => setLabel(e.target.value)}
                  placeholder="Name, like Bybit"
                  className="h-11 w-32 rounded-[10px] border border-white/10 bg-white/4 px-3 text-[13px] outline-none focus:border-white/25"
                />
                <label htmlFor="source-site" className="sr-only">
                  Apps or sites
                </label>
                <input
                  id="source-site"
                  value={site}
                  onChange={(e) => setSite(e.target.value)}
                  placeholder="Apps or sites, like Bybit, bybit.com"
                  className="h-11 min-w-0 flex-1 rounded-[10px] border border-white/10 bg-white/4 px-3 text-[13px] outline-none focus:border-white/25"
                />
                <button
                  type="submit"
                  disabled={!label.trim() || !site.trim()}
                  className="h-11 rounded-[10px] bg-white/10 px-3.5 text-[13px] font-medium disabled:opacity-40"
                >
                  Add
                </button>
              </form>
            </Group>
          </div>

          {error && (
            <p role="alert" className="text-[13px] text-alert">
              {error}
            </p>
          )}
          <p className="text-[11px] text-text-muted">
            Halo never places trades, holds exchange keys, or gives trading advice.
          </p>
        </div>
        {removing !== null && settings.sources[removing] && (
          <ConfirmDialog
            title="Stop tracking this?"
            message={`Time on ${settings.sources[removing].label} won't count as chart time any more. Time already logged stays.`}
            confirmLabel="Remove"
            onCancel={() => setRemoving(null)}
            onConfirm={() => {
              const i = removing;
              setRemoving(null);
              save({ sources: settings.sources.filter((_, j) => j !== i) });
            }}
          />
        )}
      </div>
    </div>
  );
}
