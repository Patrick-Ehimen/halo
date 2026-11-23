import type { TradingSettings, TradingView } from "../api";
import { COLOR_BY_NAME } from "../colors";
import { useTradingStore } from "../store";

const MARKERS = ["00:00", "06:00", "12:00", "18:00", "24:00"];
const OUTSIDE = "repeating-linear-gradient(45deg, #FF7A6B 0 3px, #C4594C 3px 6px)";

/** Time on charting apps and sites, and how much fell outside the window. */
export function ChartTime({
  view,
  settings,
  onEditSources,
}: {
  view: TradingView;
  settings: TradingSettings | null;
  /** Opens desk settings at the charting apps and sites. */
  onEditSources: () => void;
}) {
  const saveSettings = useTradingStore((s) => s.saveSettings);
  const chart = view.chart;
  const maxWeek = Math.max(1, ...chart.week.map((d) => d.minutes));
  const windowName = view.clocks.find((c) => c.mine)?.name ?? "";
  // The sessions open right now, and where they sit on today's strip.
  const openNow = view.timeline
    .filter((r) => view.clocks.some((c) => c.name === r.name && c.open))
    .flatMap((r) =>
      r.segments
        .filter((g) => g.start <= view.nowPermille && view.nowPermille < g.end)
        .map((g) => ({ name: r.name, ...g })),
    );

  return (
    <section
      aria-label="Chart time outside your window"
      className="flex min-w-0 shrink-0 flex-col gap-3.5 rounded-2xl border border-card-border bg-card px-[18px] py-4"
    >
      <div className="flex flex-col gap-0.5">
        <h2 className="text-[13px] font-semibold text-[#FF9A8D]">Chart time outside your window</h2>
        <button
          type="button"
          onClick={onEditSources}
          title="Choose the apps and sites to track"
          className="self-start text-left text-xs text-text-secondary underline-offset-2 hover:text-text hover:underline"
        >
          {chart.trackingLabel}
        </button>
        {chart.browserBlocked && (
          <button
            type="button"
            onClick={onEditSources}
            className="self-start text-left text-xs text-alert underline-offset-2 hover:underline"
          >
            Browser sites aren't counted: Automation is off
          </button>
        )}
      </div>

      <div className="flex items-baseline gap-2.5">
        <span className="text-[34px] font-semibold tracking-[-0.02em] text-[#FF9A8D] tabular-nums">
          {chart.outsideLabel}
        </span>
        <span className="text-[13px] text-text-secondary tabular-nums">{chart.totalLabel}</span>
      </div>

      <div className="flex flex-col gap-1.5">
        <div
          role="img"
          aria-label="Today's chart sessions on a 24-hour strip"
          className="relative h-[22px] overflow-hidden rounded-md bg-white/5"
        >
          {openNow.map((o) => (
            <span
              key={`open-${o.name}`}
              className="absolute inset-y-0 border-t-2"
              style={{
                left: `${o.start / 10}%`,
                width: `${(o.end - o.start) / 10}%`,
                background: `${COLOR_BY_NAME[o.name] ?? "#8A8E9E"}26`,
                borderColor: COLOR_BY_NAME[o.name] ?? "#8A8E9E",
              }}
            />
          ))}
          {chart.window.map((w, i) => (
            <span
              key={`w${i}`}
              className="absolute inset-y-0 border-x border-dashed border-shield/60 bg-shield/8"
              style={{ left: `${w.start / 10}%`, width: `${(w.end - w.start) / 10}%` }}
            />
          ))}
          {chart.segments.map((g, i) => (
            <span
              key={i}
              className="absolute inset-y-1 min-w-[3px] rounded-[3px]"
              style={{
                left: `${g.start / 10}%`,
                width: `${(g.end - g.start) / 10}%`,
                background: g.inside ? "var(--halo-shield)" : OUTSIDE,
              }}
            />
          ))}
        </div>
        <div className="flex justify-between text-[10px] text-text-muted tabular-nums">
          {MARKERS.map((m) => (
            <span key={m}>{m}</span>
          ))}
        </div>
        <div className="flex flex-wrap gap-x-3.5 gap-y-1 text-[11px] text-text-secondary">
          <span className="flex items-center gap-1.5">
            <span className="size-2.5 rounded-[3px] border border-dashed border-shield/70 bg-shield/10" />
            Your window{windowName && ` · ${windowName}`}
          </span>
          {openNow.map((o) => (
            <span key={`legend-${o.name}`} className="flex items-center gap-1.5">
              <span
                className="size-2.5 rounded-[3px]"
                style={{ background: COLOR_BY_NAME[o.name] ?? "#8A8E9E" }}
              />
              {o.name} open now
            </span>
          ))}
          <span className="flex items-center gap-1.5">
            <span className="size-2.5 rounded-[3px] bg-shield" />
            In window
          </span>
          <span className="flex items-center gap-1.5">
            <span className="size-2.5 rounded-[3px]" style={{ background: OUTSIDE }} />
            Outside window
          </span>
        </div>
      </div>

      <div className="flex flex-wrap gap-4">
        <div className="flex flex-[1_1_140px] flex-col gap-1.5">
          <span className="text-[11px] text-text-muted">Outside, last 7 days</span>
          <div
            role="img"
            aria-label="Minutes charting outside the window each day this week"
            className="flex h-[60px] items-end gap-1.5"
          >
            {chart.week.map((d, i) => (
              <span
                key={i}
                title={`${d.minutes} min`}
                className="flex h-full flex-1 flex-col items-center justify-end gap-1"
              >
                <span
                  className="w-full rounded"
                  style={{
                    height: `${Math.max(2, Math.round((d.minutes / maxWeek) * 44))}px`,
                    background: d.today ? "#FF7A6B" : "rgba(255,122,107,0.35)",
                  }}
                />
                <span className="text-[10px] text-text-muted">{d.label}</span>
              </span>
            ))}
          </div>
        </div>
        <div className="flex flex-[1_1_140px] flex-col gap-1.5">
          <span className="text-[11px] text-text-muted">Where it went</span>
          {chart.sources.length === 0 ? (
            <span className="text-xs text-text-muted">Nothing outside today</span>
          ) : (
            <>
              <div
                role="img"
                aria-label={`Share of outside time: ${chart.sources.map((s) => `${s.label} ${s.share}`).join(", ")}`}
                className="flex h-2 gap-0.5 overflow-hidden rounded-full"
              >
                {chart.sources.map((s) => (
                  <span
                    key={s.label}
                    className="h-full"
                    style={{ flex: Math.max(s.permille, 8), background: s.color }}
                  />
                ))}
              </div>
              {chart.sources.map((s) => (
                <div key={s.label} className="flex items-center gap-2 text-xs">
                  <span
                    aria-hidden="true"
                    className="size-2 shrink-0 rounded-full"
                    style={{ background: s.color }}
                  />
                  <span className="min-w-0 flex-1 truncate">{s.label}</span>
                  <span className="text-text-secondary tabular-nums">{s.outside}</span>
                  <span className="w-9 text-right font-medium tabular-nums">{s.share}</span>
                </div>
              ))}
            </>
          )}
        </div>
      </div>

      {settings && (
        <div className="flex items-center gap-3 border-t border-white/6 pt-2.5">
          <span className="flex flex-1 flex-col gap-0.5">
            <span className="text-[13px] font-medium">
              Nudge me after {settings.nudgeMinutes} min
            </span>
            <span className="text-xs text-text-muted">
              A quiet notification when you chart outside your window
            </span>
          </span>
          <button
            type="button"
            role="switch"
            aria-checked={settings.nudge}
            aria-label={`Nudge me after ${settings.nudgeMinutes} minutes`}
            onClick={() => void saveSettings({ ...settings, nudge: !settings.nudge })}
            className={`flex h-7 w-[46px] shrink-0 rounded-full p-[3px] transition-colors ${
              settings.nudge ? "justify-end bg-focus" : "justify-start bg-white/14"
            }`}
          >
            <span className="size-[22px] rounded-full bg-white" />
          </button>
        </div>
      )}
    </section>
  );
}
