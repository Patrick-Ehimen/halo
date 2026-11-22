import { useEffect } from "react";
import { appApi } from "../../features/app/api";
import { SmallRing } from "../../features/focus/components/SmallRing";
import { formatClock } from "../../features/focus/format";
import { useFocusStore } from "../../features/focus/store";

/** One simple prompt per break kind. Guided breaks arrive in phase 7. */
const PROMPTS = {
  shortBreak: {
    title: "Look at something 6 metres away for 20 seconds",
    next: "Then roll your shoulders and stand up for a moment.",
  },
  longBreak: {
    title: "Get up and move",
    next: "Water, daylight, a short walk. Leave the screen behind.",
  },
} as const;

/** Covers a display during a break. One window per display. */
export function BreakOverlayWindow() {
  const { snapshot, connect } = useFocusStore();
  const actions = useFocusStore();

  useEffect(() => connect(), [connect]);

  const phase = snapshot?.phase === "longBreak" ? "longBreak" : "shortBreak";
  const prompt = PROMPTS[phase];

  return (
    <div className="flex h-screen flex-col items-center justify-center gap-6 bg-[rgba(8,14,26,0.86)] px-8 text-center text-text backdrop-blur-[40px] backdrop-saturate-150">
      <div className="relative flex size-[220px] items-center justify-center">
        <SmallRing
          size={220}
          stroke={8}
          progressPermille={snapshot?.progressPermille ?? 0}
          accent="break"
        />
        <span
          role="timer"
          className="absolute text-[64px] leading-none font-light tracking-[-0.03em] tabular-nums"
        >
          {snapshot ? formatClock(snapshot.remainingMs, "up") : "--:--"}
        </span>
      </div>
      <div className="max-w-[560px]">
        <p className="text-sm font-medium text-break">
          {phase === "longBreak" ? "Long break" : "Short break"}
        </p>
        <h1 className="mt-1 text-2xl font-semibold text-balance">{prompt.title}</h1>
        <p className="mt-2 text-[15px] text-text-secondary">{prompt.next}</p>
      </div>
      <div className="flex gap-3">
        <button
          type="button"
          onClick={() => void actions.end()}
          className="h-12 rounded-full border border-white/12 bg-white/6 px-6 text-sm font-medium text-text hover:bg-white/10"
        >
          Skip break
        </button>
        <button
          type="button"
          onClick={() => void appApi.closeBreakOverlay()}
          className="h-12 rounded-full bg-break px-6 text-sm font-semibold text-break-ink"
        >
          I’m stretching
        </button>
      </div>
      <p className="text-xs text-text-muted">The break keeps counting in the menu bar.</p>
    </div>
  );
}
