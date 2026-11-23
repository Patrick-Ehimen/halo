import type { StatsView } from "../features/stats/api";

const day = (label: string, level: number | null) => ({ label, level });

/** A week with some focus, as Rust would send it. */
export const statsWeek: StatsView = {
  rangeLabel: "23–29 Nov",
  kpis: [
    { label: "Focus score", value: "82", detail: "+6 vs last week", good: true },
    { label: "Focused this week", value: "21h 40m", detail: "+3h 05m vs last week", good: true },
    { label: "Sessions", value: "31", detail: "Average rating good (3.2 of 4)", good: false },
    { label: "Distractions", value: "12", detail: "−4 vs last week", good: true },
  ],
  heatmap: Array.from({ length: 24 }, (_, w) =>
    Array.from({ length: 7 }, (_, d) => day(`w${w}d${d}`, w === 23 && d > 0 ? null : (w + d) % 5)),
  ),
  hours: Array.from({ length: 24 }, (_, h) => (h >= 9 && h < 11 ? 120 : h > 6 && h < 20 ? 30 : 0)),
  bestHours: "09:00–11:00",
  projects: [
    { name: "meridian-rs", color: "#F2B544", label: "9h 06m", permille: 600 },
    { name: "No project", color: "#8A8E9E", label: "6h 04m", permille: 400 },
  ],
  achievements: [
    {
      id: "deep-diver",
      name: "Deep diver",
      description: "4h focus in one day",
      progress: 240,
      target: 240,
      status: "Earned 14 Nov 2026",
      earned: true,
    },
    {
      id: "unshakeable",
      name: "Unshakeable",
      description: "10 strict sessions",
      progress: 6,
      target: 10,
      status: "6 of 10",
      earned: false,
    },
  ],
  grove: Array.from({ length: 14 }, (_, i) => ({
    label: ["Tue", "Wed", "Thu", "Fri", "Sat", "Sun", "Mon"][i % 7] ?? "Mon",
    stage: i === 13 ? "sapling" : i % 3 === 0 ? "bloom" : "tree",
    today: i === 13,
  })),
};
