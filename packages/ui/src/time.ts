/**
 * Reads what was typed as a 24-hour time: "15:30", "1530", "9:05", "905",
 * "9" or "9.30". Returns `HH:MM`, "" for empty, or null when it isn't a time.
 */
export function parseTime(text: string): string | null {
  const t = text.trim();
  if (!t) return "";
  const m = /^(\d{1,2})(?:[:.h ]?(\d{2}))?$/.exec(t);
  if (!m) return null;
  const h = Number(m[1]);
  const min = Number(m[2] ?? 0);
  if (h > 23 || min > 59) return null;
  return `${String(h).padStart(2, "0")}:${String(min).padStart(2, "0")}`;
}
