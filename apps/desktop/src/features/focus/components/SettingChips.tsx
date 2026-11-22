import type { TimerSnapshot } from "../api";
import { formatClock } from "../format";

export function SettingChips({ snapshot: s }: { snapshot: TimerSnapshot }) {
  if (s.phase !== "focus") return null;
  const chips: string[] = [];
  if (s.kind !== "flowtime") {
    chips.push(
      s.overflowEnabled
        ? `Overflow on — keeps counting past ${formatClock(s.plannedMs)}`
        : "Overflow off — ends at the goal",
    );
  }
  if (s.pausesLeft === null) chips.push("Unlimited pauses");
  else chips.push(`${s.pausesLeft} ${s.pausesLeft === 1 ? "pause" : "pauses"} left`);
  if (s.autoStartBreaks) chips.push("Break auto-starts");

  return (
    <ul className="flex flex-wrap justify-center gap-2">
      {chips.map((c) => (
        <li
          key={c}
          className="rounded-full border border-white/7 bg-control px-2.5 py-[5px] text-xs text-text-secondary"
        >
          {c}
        </li>
      ))}
    </ul>
  );
}
