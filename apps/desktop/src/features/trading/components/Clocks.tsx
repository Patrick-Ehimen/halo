import type { TradingView } from "../api";
import { COLOR_BY_NAME, MARKET_COLORS } from "../colors";

const MARKERS = ["00:00", "06:00", "12:00", "18:00", "24:00"];

/** Session clocks and the 24-hour strip, all in the user's zone. */
export function Clocks({ view }: { view: TradingView }) {
  return (
    <section
      aria-label="Session clocks"
      className="flex flex-col gap-4 rounded-2xl border border-card-border bg-white/4 px-5 py-[18px]"
    >
      <div className="flex flex-wrap items-baseline gap-x-4 gap-y-2.5">
        <h2 className="text-[13px] font-semibold text-shield">Session clocks</h2>
        <span className="text-xl font-medium tracking-[-0.01em]">{view.headline}</span>
        {view.headlineTime && (
          <span role="timer" className="text-xl font-light text-shield tabular-nums">
            {view.headlineTime}
          </span>
        )}
        {view.overlap && (
          <span className="rounded-full bg-focus/16 px-2.5 py-[3px] text-[11px] font-semibold text-[#FFE2A6]">
            London–New York overlap · peak liquidity
          </span>
        )}
        <span className="ml-auto text-[11px] text-text-muted">
          Crypto trades 24/7: these are liquidity windows
        </span>
      </div>

      <div className="grid grid-cols-[repeat(auto-fit,minmax(190px,1fr))] gap-3">
        {view.clocks.map((c) => (
          <div
            key={c.market}
            className={`flex flex-col gap-1 rounded-card px-4 py-3.5 ${
              c.open
                ? "border border-shield/30 bg-shield/6"
                : "border border-white/7 bg-white/[0.035]"
            }`}
          >
            <div className="flex items-center justify-between gap-2">
              <span className="flex min-w-0 items-center gap-2 truncate text-[13px] font-semibold">
                <span
                  aria-hidden="true"
                  className="size-2 rounded-full"
                  style={{ background: MARKET_COLORS[c.market] }}
                />
                {c.name}
                {c.mine && <span className="font-normal text-text-muted"> · your window</span>}
              </span>
              <span
                className={`rounded-full px-2 py-[3px] text-[11px] font-semibold ${
                  c.open ? "bg-shield/16 text-[#C6F2DA]" : "bg-white/6 text-text-secondary"
                }`}
              >
                {c.status}
              </span>
            </div>
            <span className="text-[30px] font-light tracking-[-0.02em] tabular-nums">
              {c.localTime}
            </span>
            <span className="text-xs text-text-secondary tabular-nums">
              {c.range} {view.zoneLabel}
            </span>
            <span className="text-xs font-medium text-[#E6E7EC] tabular-nums">{c.countdown}</span>
          </div>
        ))}
      </div>

      <div
        role="img"
        aria-label={`24-hour timeline in ${view.zoneLabel} time showing when each session is open`}
        className="flex flex-col gap-1.5"
      >
        <div className="relative flex flex-col gap-1.5 pl-[74px]">
          {view.timeline.map((r) => {
            const open = view.clocks.some((c) => c.name === r.name && c.open);
            return (
              <div key={r.name} className="relative h-3.5 rounded bg-white/4">
                <span
                  className={`absolute -top-px -left-[74px] text-[11px] ${open ? "font-semibold text-text" : "text-text-secondary"}`}
                >
                  {r.name}
                </span>
                {r.segments.map((g, i) => {
                  // Only the piece under "Now" is the session that's open.
                  const live = open && g.start <= view.nowPermille && view.nowPermille < g.end;
                  return (
                    <span
                      key={i}
                      className={`absolute inset-y-0 rounded ${live ? "ring-2 ring-shield/70" : ""}`}
                      style={{
                        left: `${g.start / 10}%`,
                        width: `${(g.end - g.start) / 10}%`,
                        background: COLOR_BY_NAME[r.name],
                        opacity: live ? 1 : 0.6,
                      }}
                    />
                  );
                })}
              </div>
            );
          })}
          <div
            className="absolute inset-y-0 w-0.5 rounded-sm bg-focus"
            style={{ left: `calc(74px + (100% - 74px) * ${view.nowPermille / 1000})` }}
          >
            <span className="absolute -top-[18px] -left-[22px] w-11 text-center text-[10px] font-semibold text-focus">
              Now
            </span>
          </div>
        </div>
        <div className="flex justify-between pl-[74px] text-[10px] text-text-muted tabular-nums">
          {MARKERS.map((m) => (
            <span key={m}>{m}</span>
          ))}
        </div>
      </div>
    </section>
  );
}
