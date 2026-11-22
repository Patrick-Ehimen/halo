import type { PresetId, TimerSnapshot } from "../focus/api";

export type PaletteAction =
  | { type: "start"; preset: PresetId; intention: string }
  | { type: "startPending" }
  | { type: "pause" }
  | { type: "resume" }
  | { type: "extend" }
  | { type: "end" }
  | { type: "toggleMini" }
  | { type: "openMain" }
  | { type: "openSettings" }
  | { type: "logSession" };

export type PaletteItem = { id: string; label: string; hint: string; action: PaletteAction };

const PRESET_NAMES: Record<PresetId, string> = {
  pomodoro: "Pomodoro 25",
  deep: "Deep 50",
  flowtime: "Flowtime",
};

/** Words that pick a preset or are filler, not part of an intention. */
const PRESET_WORDS: [RegExp, PresetId][] = [
  [/^(25|pomo|pomodoro)$/, "pomodoro"],
  [/^(50|deep)$/, "deep"],
  [/^(flow|flowtime)$/, "flowtime"],
];
const FILLER = /^(focus|start|min|mins|minutes|session|for)$/;

/**
 * Reads a quick start like "focus 50 on meridian" or "deep write tests":
 * a preset word, and what to focus on (after "on", or the remaining words).
 * Returns null when the text names no preset and doesn't start with "focus".
 */
export function parseQuickStart(
  query: string,
): { preset: PresetId | null; intention: string } | null {
  const text = query.trim();
  if (!text) return null;
  const onMatch = /\bon\s+(.+)$/i.exec(text);
  const head = onMatch ? text.slice(0, onMatch.index) : text;
  let preset: PresetId | null = null;
  const rest: string[] = [];
  for (const word of head.split(/\s+/)) {
    const lower = word.toLowerCase();
    const hit = PRESET_WORDS.find(([re]) => re.test(lower));
    if (hit) preset = hit[1];
    else if (!FILLER.test(lower)) rest.push(word);
  }
  const startsWithFocus = /^focus\b/i.test(text);
  if (!preset && !startsWithFocus) return null;
  const intention = (onMatch?.[1] ?? rest.join(" ")).trim();
  return { preset, intention };
}

/** Every command that makes sense right now, before filtering. */
export function paletteItems(s: TimerSnapshot | null, miniShown: boolean): PaletteItem[] {
  const items: PaletteItem[] = [];
  const idle = s?.status === "idle";
  const onFocus = s?.phase === "focus";
  if (!s || (idle && onFocus)) {
    (["pomodoro", "deep", "flowtime"] as const).forEach((preset, i) =>
      items.push({
        id: `start:${preset}`,
        label: `Start ${PRESET_NAMES[preset]}`,
        hint: `⌘${i + 1}`,
        action: { type: "start", preset, intention: "" },
      }),
    );
  } else if (idle) {
    items.push({
      id: "start-pending",
      label: "Start break",
      hint: "",
      action: { type: "startPending" },
    });
  } else {
    items.push(
      s.status === "running"
        ? { id: "pause", label: "Pause", hint: "⌥⌘S", action: { type: "pause" } }
        : { id: "resume", label: "Resume", hint: "⌥⌘S", action: { type: "resume" } },
    );
    if (onFocus && s.kind !== "flowtime") {
      items.push({ id: "extend", label: "Add 5 minutes", hint: "", action: { type: "extend" } });
    }
    items.push({
      id: "end",
      label: onFocus ? "End session" : "Skip break",
      hint: "",
      action: { type: "end" },
    });
  }
  items.push(
    {
      id: "mini",
      label: miniShown ? "Hide mini timer" : "Show mini timer",
      hint: "⌥⌘M",
      action: { type: "toggleMini" },
    },
    { id: "open", label: "Open Halo", hint: "", action: { type: "openMain" } },
    { id: "settings", label: "Timer settings", hint: "⌘,", action: { type: "openSettings" } },
    { id: "log", label: "Log a missed session", hint: "", action: { type: "logSession" } },
  );
  return items;
}

/**
 * Filters items by the query: every word must appear in the label, and
 * labels that start with the query rank first. A quick start like
 * "focus 50 on meridian" adds a "Start … on …" item at the top.
 */
export function filterItems(
  items: PaletteItem[],
  query: string,
  s: TimerSnapshot | null,
): PaletteItem[] {
  const q = query.trim().toLowerCase();
  if (!q) return items;
  const words = q.split(/\s+/);
  const matches = items
    .filter((item) => words.every((w) => item.label.toLowerCase().includes(w)))
    .sort(
      (a, b) =>
        Number(b.label.toLowerCase().startsWith(q)) - Number(a.label.toLowerCase().startsWith(q)),
    );

  const canStart = !s || (s.status === "idle" && s.phase === "focus");
  const quick = canStart ? parseQuickStart(query) : null;
  if (quick && (quick.intention || quick.preset)) {
    const preset = quick.preset ?? (s && s.kind !== "custom" ? s.kind : "pomodoro");
    const label = quick.intention
      ? `Start ${PRESET_NAMES[preset]} on “${quick.intention}”`
      : `Start ${PRESET_NAMES[preset]}`;
    const startItem: PaletteItem = {
      id: "quick-start",
      label,
      hint: "Enter",
      action: { type: "start", preset, intention: quick.intention },
    };
    return [startItem, ...matches.filter((m) => m.id !== `start:${preset}`)];
  }
  return matches;
}
