import type { PaceView } from "../api";

/** Colours for goals and projects. Matches `COLORS` in Rust. */
export const COLORS = ["#F2B544", "#6CB6FF", "#C59CFF", "#5BD69A", "#FF9F7A", "#7FD1C9"];

export const PACE: Record<PaceView, { label: string; className: string }> = {
  done: { label: "Done", className: "bg-shield/12 text-shield" },
  ahead: { label: "Ahead", className: "bg-shield/12 text-shield" },
  onPace: { label: "On pace", className: "bg-shield/12 text-shield" },
  behind: { label: "Behind", className: "bg-alert/12 text-alert" },
  overdue: { label: "Overdue", className: "bg-alert/12 text-alert" },
  noDeadline: { label: "No date", className: "bg-white/6 text-text-muted" },
};

export function paceLabel(p: PaceView): string {
  return PACE[p].label;
}
