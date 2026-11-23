import { HaloMark } from "@halo/ui";
import type { ReactNode } from "react";
import type { TodayFocus } from "../../features/focus/api";
import type { GoalView, ProjectView } from "../../features/plan/api";
import { Dot, GoalRing } from "../../features/plan/components/ui";

export type Screen = "focus" | "plan" | "goals" | "stats" | "trading";

type Item = { label: string; screen?: Screen; icon: (color: string) => ReactNode };

const stroke = (color: string) => ({
  width: 18,
  height: 18,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: color,
  strokeWidth: 1.75,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
  "aria-hidden": true,
});

const ITEMS: Item[] = [
  {
    label: "Focus",
    screen: "focus",
    icon: (c) => (
      <svg {...stroke(c)}>
        <circle cx="12" cy="13" r="8" />
        <path d="M12 9v4l2.5 2M10 2h4" />
      </svg>
    ),
  },
  {
    label: "Today plan",
    screen: "plan",
    icon: (c) => (
      <svg {...stroke(c)}>
        <rect x="3" y="5" width="18" height="16" rx="2" />
        <path d="M3 10h18M8 3v4M16 3v4" />
      </svg>
    ),
  },
  {
    label: "Goals",
    screen: "goals",
    icon: (c) => (
      <svg {...stroke(c)}>
        <path d="M5 21V4" />
        <path d="M5 4h11l-2 4 2 4H5" />
      </svg>
    ),
  },
  {
    label: "Stats",
    screen: "stats",
    icon: (c) => (
      <svg {...stroke(c)}>
        <path d="M4 20V4M4 20h16M9 16v-5M14 16V8M19 16v-3" />
      </svg>
    ),
  },
  {
    label: "Shield",
    icon: (c) => (
      <svg {...stroke(c)}>
        <path d="M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6z" />
      </svg>
    ),
  },
  {
    label: "Trading desk",
    screen: "trading",
    icon: (c) => (
      <svg {...stroke(c)}>
        <path d="M7 4v16M17 4v16M5 8h4v6H5zM15 10h4v5h-4z" />
      </svg>
    ),
  },
];

type SidebarProps = {
  version: string | null;
  screen: Screen;
  onNavigate: (screen: Screen, goalId?: string) => void;
  goals: GoalView[];
  projects: ProjectView[];
  today: TodayFocus | null;
};

export function Sidebar({ version, screen, onNavigate, goals, projects, today }: SidebarProps) {
  const active = goals.filter((g) => g.status === "active");
  const loose = projects.filter((p) => !p.goalId);
  return (
    <nav
      aria-label="Main"
      data-tauri-drag-region
      className="flex w-[220px] shrink-0 flex-col gap-0.5 overflow-y-auto border-r border-hairline bg-white/[0.025] px-3 pt-11 pb-4"
    >
      <div className="flex items-center gap-2.5 px-2 pb-[18px]">
        <HaloMark size={26} />
        <span className="text-[19px] font-semibold tracking-[-0.03em]">halo</span>
      </div>
      <p className="mx-2.5 my-1.5 text-[11px] font-semibold tracking-[0.06em] text-text-muted uppercase">
        Workspace
      </p>
      {ITEMS.map((item) => {
        const current = item.screen === screen;
        if (!item.screen) {
          return (
            <span
              key={item.label}
              aria-disabled="true"
              title="Coming in a later phase"
              className="flex min-h-9 items-center gap-2.5 rounded-lg px-2.5 text-[13px] font-medium text-text-secondary opacity-60"
            >
              {item.icon("currentColor")}
              {item.label}
            </span>
          );
        }
        const to = item.screen;
        return (
          <button
            key={item.label}
            type="button"
            aria-current={current ? "page" : undefined}
            onClick={() => onNavigate(to)}
            className={`flex min-h-9 items-center gap-2.5 rounded-lg px-2.5 text-left text-[13px] font-medium ${
              current
                ? "bg-white/8 text-text"
                : "text-text-secondary hover:bg-white/4 hover:text-text"
            }`}
          >
            {item.icon(current ? "var(--halo-focus)" : "currentColor")}
            {item.label}
          </button>
        );
      })}

      {(active.length > 0 || loose.length > 0) && (
        <p className="mx-2.5 mt-[18px] mb-1.5 text-[11px] font-semibold tracking-[0.06em] text-text-muted uppercase">
          Goals
        </p>
      )}
      {active.map((g) => (
        <div key={g.id}>
          <button
            type="button"
            onClick={() => onNavigate("goals", g.id)}
            className="flex min-h-8 w-full items-center gap-2.5 rounded-lg px-2.5 text-left text-[13px] font-medium text-text hover:bg-white/4"
          >
            <GoalRing size={16} stroke={2.6} percent={g.percent} color={g.color} />
            <span className="truncate">{g.title}</span>
          </button>
          {projects
            .filter((p) => p.goalId === g.id)
            .map((p) => (
              <button
                key={p.id}
                type="button"
                onClick={() => onNavigate("plan")}
                className="flex min-h-7 w-full items-center gap-2 rounded-lg pr-2.5 pl-9 text-left text-xs text-text-secondary hover:bg-white/4"
              >
                <Dot color={p.color} />
                <span className="truncate">{p.name}</span>
              </button>
            ))}
        </div>
      ))}
      {loose.length > 0 && (
        <div>
          <p className="flex min-h-8 items-center gap-2.5 px-2.5 text-[13px] font-medium text-text-secondary">
            <span className="size-4 rounded-full border-2 border-white/14" aria-hidden="true" />
            No goal
          </p>
          {loose.map((p) => (
            <button
              key={p.id}
              type="button"
              onClick={() => onNavigate("plan")}
              className="flex min-h-7 w-full items-center gap-2 rounded-lg pr-2.5 pl-9 text-left text-xs text-text-secondary hover:bg-white/4"
            >
              <Dot color={p.color} />
              <span className="truncate">{p.name}</span>
            </button>
          ))}
        </div>
      )}

      <div className="mt-auto flex flex-col gap-3 pt-4">
        {today && (
          <button
            type="button"
            onClick={() => onNavigate("stats")}
            className="flex flex-col gap-1 rounded-xl border border-focus/20 bg-focus/8 p-3 text-left hover:bg-focus/12"
          >
            <span className="flex items-center gap-2 text-[13px] font-semibold tabular-nums">
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="var(--halo-focus)"
                strokeWidth="1.9"
                strokeLinecap="round"
                strokeLinejoin="round"
                aria-hidden="true"
              >
                <path d="M12 2c1 4 6 6 6 12a6 6 0 0 1-12 0c0-3 2-5 3-6 0 2 1 3 2 3 0-4-1-6 1-9z" />
              </svg>
              {today.streak === 0 ? "No streak yet" : `${today.streak}-day streak`}
            </span>
            <span className="text-xs text-text-secondary tabular-nums">
              {today.streak > 0 && !today.streakToday
                ? "Focus today to keep it"
                : `Best: ${today.streakLongest} ${today.streakLongest === 1 ? "day" : "days"}`}
            </span>
          </button>
        )}
        <p className="px-2.5 text-[11px] text-text-muted tabular-nums">
          {version ? `v${version}` : ""}
        </p>
      </div>
    </nav>
  );
}
