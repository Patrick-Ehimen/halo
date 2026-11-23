import { useEffect } from "react";
import { tradingApi } from "../api";
import { COLOR_BY_NAME, MARKET_COLORS } from "../colors";
import { useTradingStore } from "../store";

/** The Trading tab of the menu bar popover: a compact desk. */
export function PopoverTrading() {
  const { view, error, connect, run } = useTradingStore();
  useEffect(() => connect(), [connect]);

  if (!view) {
    return <p className="px-1 text-[13px] text-text-muted">{error ?? "Loading the desk"}</p>;
  }
  const next = view.checklist.find((c) => !c.checked);
  const total = view.checklist.length;
  const logged = view.headCheck;

  return (
    <div className="flex flex-col gap-3">
      <section
        aria-label="Your window"
        className="rounded-card border border-card-border bg-card p-3.5"
      >
        <div className="flex items-center justify-between text-xs">
          <span className="font-medium text-text-secondary">{view.headline}</span>
          <span className="text-text-muted tabular-nums">
            {view.zoneLabel} {view.nowLabel}
          </span>
        </div>
        <p
          role="timer"
          className={`text-[30px] leading-tight font-light tracking-[-0.02em] tabular-nums ${view.windowOpen ? "text-shield" : ""}`}
        >
          {view.headlineTime || "Weekend"}
        </p>
        <div className="mt-2 flex flex-col gap-1" aria-hidden="true">
          {view.timeline.map((r) => (
            <div key={r.name} className="relative h-1.5 rounded-sm bg-white/5">
              {r.segments.map((g, i) => (
                <span
                  key={i}
                  className="absolute inset-y-0 rounded-sm"
                  style={{
                    left: `${g.start / 10}%`,
                    width: `${(g.end - g.start) / 10}%`,
                    background: COLOR_BY_NAME[r.name],
                    opacity: r.mine ? 1 : 0.75,
                  }}
                />
              ))}
              <span
                className="absolute -inset-y-0.5 w-px bg-focus"
                style={{ left: `${view.nowPermille / 10}%` }}
              />
            </div>
          ))}
        </div>
        <ul className="mt-2.5 flex flex-col gap-1 text-xs">
          {view.clocks.map((c) => (
            <li key={c.market} className="flex items-center gap-2">
              <span
                className="size-2 rounded-full"
                style={{ background: MARKET_COLORS[c.market], opacity: c.open ? 1 : 0.35 }}
              />
              <span className={`flex-1 ${c.mine ? "font-semibold" : ""}`}>{c.name}</span>
              <span className="text-text-secondary tabular-nums">{c.localTime}</span>
              <span className="w-14 text-right text-text-muted">{c.status}</span>
            </li>
          ))}
        </ul>
      </section>

      <section
        aria-label="Pre-trade checklist"
        className="rounded-card border border-card-border bg-card p-3.5"
      >
        <div className="flex items-center justify-between text-xs">
          <span className="font-medium">Pre-trade checklist</span>
          <span className="text-text-secondary tabular-nums">
            {view.checklistDone} of {total}
          </span>
        </div>
        <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-white/8">
          <div
            className="h-full rounded-full bg-shield"
            style={{ width: `${total === 0 ? 0 : (view.checklistDone / total) * 100}%` }}
          />
        </div>
        {next ? (
          <button
            type="button"
            onClick={() => void run(() => tradingApi.setChecked(next.id, true))}
            className="mt-2 flex min-h-11 w-full items-center gap-2.5 rounded-lg px-1 text-left text-[13px] hover:bg-white/4"
          >
            <span className="size-4 shrink-0 rounded border-[1.5px] border-white/28" />
            <span className="min-w-0 flex-1 truncate">Next: {next.label}</span>
            <span className="text-xs font-medium text-shield">Tick</span>
          </button>
        ) : (
          <p className="mt-2 text-xs text-text-secondary">
            {total === 0 ? "No checklist yet" : "All done for today"}
          </p>
        )}
      </section>

      <section
        aria-label="Head check"
        className="rounded-card border border-card-border bg-card p-3.5"
      >
        <div className="flex items-center justify-between text-xs">
          <span className="font-medium">Head check</span>
          <span className="text-text-secondary">
            {logged
              ? logged.lockedToday
                ? "Sitting out today"
                : logged.atLabel
              : "How clear is your head?"}
          </span>
        </div>
        <div
          role="group"
          aria-label="Rate your state from 1 to 5"
          className="mt-2 grid grid-cols-5 gap-1.5"
        >
          {[1, 2, 3, 4, 5].map((n) => {
            const on = logged?.score === n;
            return (
              <button
                key={n}
                type="button"
                aria-pressed={on}
                disabled={logged?.lockedToday}
                onClick={() =>
                  void run(() =>
                    on
                      ? tradingApi.clearHeadCheck()
                      : tradingApi.headCheck(n, logged?.factors ?? []),
                  )
                }
                className={`h-11 rounded-lg border text-sm font-semibold disabled:opacity-50 ${
                  on
                    ? n <= 2
                      ? "border-alert bg-alert/18 text-[#FFD1CB]"
                      : n === 3
                        ? "border-focus bg-focus/18 text-[#FFE2A6]"
                        : "border-shield bg-shield/16 text-[#C6F2DA]"
                    : "border-white/10 bg-white/4 text-text-secondary"
                }`}
              >
                {n}
              </button>
            );
          })}
        </div>
      </section>

      <section
        aria-label="Chart time"
        className="flex items-center justify-between rounded-card border border-card-border bg-card px-3.5 py-3"
      >
        <span className="flex flex-col">
          <span className="text-[13px]">Charts outside your window today</span>
          <span className="text-xs text-text-muted">{view.gate.label}</span>
        </span>
        <span className="text-[15px] font-semibold text-alert tabular-nums">
          {view.chart.outsideLabel}
        </span>
      </section>
      {error && (
        <p role="status" className="text-xs text-alert">
          {error}
        </p>
      )}
    </div>
  );
}
