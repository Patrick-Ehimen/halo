import type { PresetId, TimerSnapshot } from "./api";

/** mm:ss, or h:mm:ss past an hour. Display only: the values come from Rust. */
export function formatClock(ms: number, round: "up" | "down" = "down"): string {
  const abs = Math.abs(ms) / 1000;
  const total = round === "up" ? Math.ceil(abs) : Math.floor(abs);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${pad(m)}:${pad(s)}`;
}

export function formatMinutes(ms: number): string {
  return `${Math.round(ms / 60000)} min`;
}

/** "3h 20m", "45m". */
export function formatHoursMinutes(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

export const PRESETS: { id: PresetId; label: string }[] = [
  { id: "pomodoro", label: "Pomodoro 25" },
  { id: "deep", label: "Deep 50" },
  { id: "flowtime", label: "Flowtime" },
];

/** What the big clock shows, and the labels around it. */
export function describe(s: TimerSnapshot): {
  clock: string;
  label: string;
  sub: string;
  accent: "focus" | "break";
} {
  const isBreak = s.phase !== "focus";
  if (isBreak) {
    const label = s.phase === "longBreak" ? "Long break" : "Short break";
    return {
      clock: formatClock(s.status === "idle" ? s.plannedMs : s.remainingMs, "up"),
      label: s.status === "paused" ? "Paused" : label,
      sub: `of a ${formatMinutes(s.plannedMs)} break`,
      accent: "break",
    };
  }
  if (s.kind === "flowtime") {
    return {
      clock: formatClock(s.elapsedMs),
      label: s.status === "paused" ? "Paused" : "Flowtime",
      sub:
        s.status === "idle"
          ? "Your break grows as you focus"
          : `Break earned: ${formatMinutes(s.earnedBreakMs)}`,
      accent: "focus",
    };
  }
  if (s.overflowing) {
    return {
      clock: `+${formatClock(s.remainingMs)}`,
      label: s.status === "paused" ? "Paused" : "Overflow",
      sub: `Past your ${formatMinutes(s.plannedMs)} goal`,
      accent: "break",
    };
  }
  return {
    clock: formatClock(s.remainingMs, "up"),
    label: s.status === "paused" ? "Paused" : "Focus",
    sub: `of a ${formatMinutes(s.plannedMs)} session`,
    accent: "focus",
  };
}

export function presetOf(s: TimerSnapshot): PresetId {
  return s.kind === "custom" ? "pomodoro" : s.kind;
}
