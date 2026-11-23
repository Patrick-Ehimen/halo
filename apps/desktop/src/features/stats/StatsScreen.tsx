import { useEffect } from "react";
import type { StatsRange, StatsView } from "./api";
import { Tree } from "./components/Tree";
import { useStatsStore } from "./store";

const RANGES: { id: StatsRange; label: string }[] = [
  { id: "day", label: "Day" },
  { id: "week", label: "Week" },
  { id: "month", label: "Month" },
  { id: "year", label: "Year" },
];

/** Heatmap shades, none to goal met. */
const SHADES = [
  "rgba(255,255,255,0.06)",
  "rgba(242,181,68,0.22)",
  "rgba(242,181,68,0.42)",
  "rgba(242,181,68,0.68)",
  "var(--halo-focus)",
];

const card =
  "flex min-w-0 flex-col gap-3 rounded-card border border-card-border bg-card px-[18px] py-4";

export function StatsScreen() {
  const { range, view, error, exported, connect, setRange, exportSessions } = useStatsStore();
  useEffect(() => connect(), [connect]);

  return (
    <>
      <header
        data-tauri-drag-region
        className="flex flex-wrap items-center gap-3 border-b border-hairline px-7 py-4"
      >
        <div data-tauri-drag-region className="flex-1">
          <h1 className="text-lg font-semibold tracking-[-0.01em]">Stats</h1>
          <p className="text-[13px] text-text-secondary">{view?.rangeLabel ?? " "}</p>
        </div>
        <div
          role="group"
          aria-label="Range"
          className="flex gap-0.5 rounded-control border border-white/7 bg-white/5 p-[3px]"
        >
          {RANGES.map((r) => (
            <button
              key={r.id}
              type="button"
              aria-pressed={range === r.id}
              onClick={() => void setRange(r.id)}
              className={`h-9 rounded-lg px-3.5 text-[13px] font-medium ${
                range === r.id ? "bg-white/12 text-focus" : "text-text-secondary hover:text-text"
              }`}
            >
              {r.label}
            </button>
          ))}
        </div>
        <button
          type="button"
          onClick={() => void exportSessions("csv")}
          className="h-11 rounded-control border border-white/10 bg-control px-3.5 text-[13px] font-medium text-text hover:bg-white/9"
        >
          Export CSV
        </button>
        <button
          type="button"
          onClick={() => void exportSessions("json")}
          className="h-11 rounded-control border border-white/10 bg-control px-3.5 text-[13px] font-medium text-text hover:bg-white/9"
        >
          Export JSON
        </button>
      </header>

      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-auto px-7 pt-[22px] pb-7">
        {(error || exported) && (
          <p
            role="status"
            className={`text-[13px] ${error ? "text-alert" : "text-text-secondary"}`}
          >
            {error ?? `Saved to ${exported}`}
          </p>
        )}
        {view ? <Body view={view} /> : !error && <p className="sr-only">Loading</p>}
      </div>
    </>
  );
}

function Body({ view }: { view: StatsView }) {
  const maxHour = Math.max(1, ...view.hours);
  const earned = view.achievements.filter((a) => a.earned);
  const open = view.achievements.filter((a) => !a.earned);
  return (
    <>
      <div className="grid grid-cols-[repeat(auto-fit,minmax(180px,1fr))] gap-3">
        {view.kpis.map((k) => (
          <div
            key={k.label}
            className="flex flex-col gap-1.5 rounded-card border border-card-border bg-card px-[18px] py-4"
          >
            <span className="text-xs text-text-secondary">{k.label}</span>
            <span className="text-[28px] font-semibold tracking-[-0.02em] tabular-nums">
              {k.value}
            </span>
            <span className={`text-xs ${k.good ? "text-shield" : "text-text-muted"}`}>
              {k.detail}
            </span>
          </div>
        ))}
      </div>

      <div className="flex flex-wrap gap-4">
        <section aria-label="Focus heatmap" className={`${card} flex-[999_1_520px]`}>
          <div className="flex items-center justify-between">
            <h2 className="text-[13px] font-semibold">Focus heatmap · last 24 weeks</h2>
            <span
              className="flex items-center gap-1 text-[11px] text-text-muted"
              aria-hidden="true"
            >
              Less
              {SHADES.map((c) => (
                <span key={c} className="size-[13px] rounded-[3px]" style={{ background: c }} />
              ))}
              More
            </span>
          </div>
          <div className="overflow-x-auto">
            <div className="flex w-max gap-[3px]" role="list" aria-label="Daily focus">
              {view.heatmap.map((week, w) => (
                <div key={w} className="flex flex-col gap-[3px]">
                  {week.map((day) => (
                    <span
                      key={day.label}
                      role="listitem"
                      title={day.label}
                      aria-label={day.label}
                      className="size-[13px] rounded-[3px]"
                      style={{
                        background: day.level === null ? "transparent" : SHADES[day.level],
                      }}
                    />
                  ))}
                </div>
              ))}
            </div>
          </div>
        </section>

        <section aria-label="Peak focus hours" className={`${card} flex-[1_1_320px]`}>
          <div className="flex items-center justify-between">
            <h2 className="text-[13px] font-semibold">Peak focus hours</h2>
            <span className="text-[11px] text-text-muted tabular-nums">
              {view.bestHours ? `Best: ${view.bestHours}` : "No sessions yet"}
            </span>
          </div>
          <div className="flex h-[110px] items-end gap-[3px]" aria-hidden="true">
            {view.hours.map((m, h) => (
              <span
                key={h}
                title={`${String(h).padStart(2, "0")}:00 · ${m} min`}
                className="flex-1 rounded-[3px_3px_2px_2px]"
                style={{
                  height: `${Math.max(m > 0 ? 4 : 2, Math.round((m / maxHour) * 100))}%`,
                  background:
                    m > 0 && m >= maxHour * 0.8 ? "var(--halo-focus)" : "rgba(242,181,68,0.30)",
                }}
              />
            ))}
          </div>
          <div className="flex justify-between text-[11px] text-text-muted tabular-nums">
            <span>00</span>
            <span>06</span>
            <span>12</span>
            <span>18</span>
            <span>23</span>
          </div>
        </section>
      </div>

      <div className="flex flex-wrap gap-4">
        <section aria-label="Time by project" className={`${card} flex-[1_1_300px]`}>
          <h2 className="text-[13px] font-semibold">Time by project</h2>
          {view.projects.length === 0 ? (
            <p className="text-[13px] text-text-muted">No focus in this range yet.</p>
          ) : (
            <>
              <div className="flex h-2.5 gap-0.5 overflow-hidden rounded-full" aria-hidden="true">
                {view.projects.map((p) => (
                  <span
                    key={p.name}
                    className="h-full"
                    style={{ flex: Math.max(p.permille, 5), background: p.color }}
                  />
                ))}
              </div>
              {view.projects.map((p) => (
                <div key={p.name} className="flex items-center gap-2 text-[13px]">
                  <span className="size-2 rounded-full" style={{ background: p.color }} />
                  <span className="flex-1 truncate">{p.name}</span>
                  <span className="text-text-secondary tabular-nums">{p.label}</span>
                </div>
              ))}
            </>
          )}
        </section>

        <section aria-label="Achievements" className={`${card} flex-[1_1_260px]`}>
          <h2 className="text-[13px] font-semibold">
            Achievements{" "}
            <span className="font-normal text-text-muted tabular-nums">
              {earned.length} of {view.achievements.length}
            </span>
          </h2>
          <div className="grid grid-cols-2 gap-2.5">
            {[...earned, ...open].map((a) => (
              <div
                key={a.id}
                title={a.status}
                className={`flex flex-col gap-0.5 rounded-[10px] px-3 py-2.5 ${
                  a.earned
                    ? "border border-focus/28 bg-focus/10"
                    : "border border-dashed border-white/16 bg-white/3"
                }`}
              >
                <span className="text-xs font-semibold">{a.name}</span>
                <span className="text-[11px] text-text-secondary">{a.description}</span>
                <span className="text-[11px] text-text-muted tabular-nums">{a.status}</span>
              </div>
            ))}
          </div>
        </section>
      </div>

      <section aria-label="Your grove" className={card}>
        <div className="flex items-center justify-between">
          <h2 className="text-[13px] font-semibold">Your grove · last 14 days</h2>
          <span className="text-[11px] text-text-muted">
            A tree a day; it blooms at your daily goal
          </span>
        </div>
        <ol className="flex flex-wrap justify-between gap-2">
          {view.grove.map((d, i) => (
            <li key={i} className="flex flex-col items-center gap-1">
              <Tree stage={d.stage} size={44} />
              <span
                className={`text-[11px] ${d.today ? "font-semibold text-focus" : "text-text-muted"}`}
              >
                {d.today ? "Today" : d.label}
              </span>
            </li>
          ))}
        </ol>
      </section>
    </>
  );
}
