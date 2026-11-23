import { useEffect, useRef, useState, type ReactNode } from "react";
import { appApi } from "../../features/app/api";
import { SmallRing } from "../../features/focus/components/SmallRing";
import { PRESETS, describe } from "../../features/focus/format";
import { useFocusStore } from "../../features/focus/store";

function Chip({
  children,
  onClick,
  primary,
  accent = "focus",
  disabled,
}: {
  children: ReactNode;
  onClick: () => void;
  primary?: boolean;
  accent?: "focus" | "break";
  disabled?: boolean;
}) {
  const look = primary
    ? accent === "focus"
      ? "bg-focus text-focus-ink font-semibold"
      : "bg-break text-break-ink font-semibold"
    : "border border-white/10 bg-control text-text font-medium hover:bg-white/9";
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className={`h-11 flex-1 rounded-control px-3 text-[13px] transition-colors disabled:cursor-not-allowed disabled:opacity-40 ${look}`}
    >
      {children}
    </button>
  );
}

/** Card width plus its shadow margin (14px each side), in logical pixels.
 *  Matches `POPOVER_WIDTH` and `POPOVER_PAD_X` in the Rust window. */
const WINDOW_WIDTH = 340 + 2 * 14;

/** The menu bar popover: the live session, quick start, and today. */
export function PopoverWindow() {
  const { snapshot, today, error, connect } = useFocusStore();
  const actions = useFocusStore();
  const rootRef = useRef<HTMLDivElement>(null);
  const [miniShown, setMiniShown] = useState(false);

  // Re-check each time the popover opens; the mini timer can change elsewhere.
  useEffect(() => {
    const refresh = () => {
      void appApi
        .miniVisible()
        .then(setMiniShown)
        .catch(() => {});
    };
    refresh();
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
  }, []);

  // The window follows the content's height, so there's no empty space.
  useEffect(() => {
    const el = rootRef.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(() => {
      void appApi.fitWindow(WINDOW_WIDTH, el.getBoundingClientRect().height).catch(() => {});
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  useEffect(() => connect(), [connect]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") void appApi.hidePopover();
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, []);

  const view = snapshot ? describe(snapshot) : null;
  const idle = snapshot?.status === "idle";
  const isBreak = snapshot ? snapshot.phase !== "focus" : false;
  const current = snapshot ? Math.min(snapshot.completedInCycle + 1, snapshot.longBreakEvery) : 0;

  return (
    // The window is transparent with no native shadow (macOS would draw it
    // square); the margin here leaves room for the card's own shadow.
    <div ref={rootRef} className="px-[14px] pt-1 pb-6">
      <div className="flex flex-col gap-3 overflow-hidden rounded-popover border border-white/10 bg-surface p-3.5 text-text shadow-[0_12px_28px_rgba(0,0,0,0.5)]">
        <div
          role="tablist"
          aria-label="Popover view"
          className="flex gap-0.5 rounded-control bg-control p-[3px]"
        >
          <button
            type="button"
            role="tab"
            aria-selected="true"
            className="h-9 flex-1 rounded-lg bg-control-selected text-[13px] font-medium"
          >
            Focus
          </button>
          <button
            type="button"
            role="tab"
            aria-selected="false"
            aria-disabled="true"
            title="The trading desk arrives in a later phase"
            className="h-9 flex-1 cursor-not-allowed rounded-lg text-[13px] font-medium text-text-muted"
          >
            Trading
          </button>
        </div>

        <section
          aria-label="Current session"
          className="rounded-card border border-card-border bg-card p-3.5"
        >
          {snapshot && view ? (
            <>
              <div className="flex items-center gap-3.5">
                <SmallRing
                  size={52}
                  stroke={5}
                  progressPermille={idle ? 0 : snapshot.progressPermille}
                  accent={view.accent}
                />
                <div className="min-w-0 flex-1">
                  <p className="text-xs font-medium text-text-secondary">
                    {idle && !isBreak ? "Ready" : view.label}
                  </p>
                  <p
                    role="timer"
                    className="text-[34px] leading-tight font-light tracking-[-0.02em] tabular-nums"
                  >
                    {view.clock}
                  </p>
                  {isBreak ? (
                    <p className="text-xs text-text-muted">{view.sub}</p>
                  ) : (
                    <>
                      <p className="truncate text-[13px] text-text-secondary">
                        {snapshot.intention || (idle ? "Pick a mode to start" : "No intention set")}
                      </p>
                      <p className="text-xs text-text-muted tabular-nums">
                        Session {current} of {snapshot.longBreakEvery}
                      </p>
                    </>
                  )}
                </div>
              </div>

              {!idle && (
                <div className="mt-3 flex gap-2">
                  {snapshot.status === "running" ? (
                    <Chip
                      primary
                      accent={isBreak ? "break" : "focus"}
                      onClick={() => void actions.pause()}
                      disabled={!isBreak && snapshot.pausesLeft === 0}
                    >
                      Pause
                    </Chip>
                  ) : (
                    <Chip
                      primary
                      accent={isBreak ? "break" : "focus"}
                      onClick={() => void actions.resume()}
                    >
                      Resume
                    </Chip>
                  )}
                  {!isBreak && snapshot.kind !== "flowtime" && (
                    <Chip onClick={() => void actions.extend(5)}>+5 min</Chip>
                  )}
                  <Chip onClick={() => void actions.end()}>{isBreak ? "Skip break" : "End"}</Chip>
                </div>
              )}
              {idle && isBreak && (
                <div className="mt-3 flex gap-2">
                  <Chip primary accent="break" onClick={() => void actions.startPending()}>
                    Start break
                  </Chip>
                </div>
              )}
            </>
          ) : (
            <p className="text-[13px] text-text-muted">{error ?? "Loading timer"}</p>
          )}
        </section>

        {idle && (
          <section aria-label="Quick start">
            <p className="mb-2 text-[11px] font-semibold tracking-[0.06em] text-text-muted uppercase">
              Quick start
            </p>
            <div className="flex gap-2">
              {PRESETS.map((p) => (
                <Chip key={p.id} onClick={() => void actions.start(p.id, "")}>
                  {p.label}
                </Chip>
              ))}
            </div>
          </section>
        )}

        <section
          aria-label="Today"
          className="flex items-center justify-between rounded-card border border-card-border bg-card px-3.5 py-3"
        >
          <span className="text-[13px] text-text-secondary">Today</span>
          <span className="text-[13px] font-medium tabular-nums">
            {today?.focusedLabel ?? "0m"} · {today?.sessions ?? 0}{" "}
            {today?.sessions === 1 ? "session" : "sessions"}
          </span>
        </section>

        {error && snapshot && (
          <p role="status" className="text-xs text-alert">
            {error}
          </p>
        )}

        <div className="flex gap-2 border-t border-hairline pt-3">
          <Chip onClick={() => void appApi.openMainWindow()}>Open Halo</Chip>
          <Chip
            onClick={() => {
              const next = !miniShown;
              setMiniShown(next);
              void appApi.setMiniVisible(next).catch(() => setMiniShown(!next));
            }}
          >
            {miniShown ? "Hide mini" : "Mini timer"}
          </Chip>
          <button
            type="button"
            onClick={() => void appApi.quit()}
            className="h-11 rounded-control px-4 text-[13px] font-medium text-text-secondary hover:text-text"
          >
            Quit
          </button>
        </div>
      </div>
    </div>
  );
}
