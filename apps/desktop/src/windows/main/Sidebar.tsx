import { HaloMark } from "@halo/ui";
import type { ReactNode } from "react";

type Item = { label: string; icon: ReactNode; current?: boolean };

const stroke = {
  width: 18,
  height: 18,
  viewBox: "0 0 24 24",
  fill: "none",
  strokeWidth: 1.75,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
  "aria-hidden": true,
};

const ITEMS: Item[] = [
  {
    label: "Focus",
    current: true,
    icon: (
      <svg {...stroke} stroke="var(--halo-focus)">
        <circle cx="12" cy="13" r="8" />
        <path d="M12 9v4l2.5 2M10 2h4" />
      </svg>
    ),
  },
  {
    label: "Today plan",
    icon: (
      <svg {...stroke} stroke="currentColor">
        <rect x="3" y="5" width="18" height="16" rx="2" />
        <path d="M3 10h18M8 3v4M16 3v4" />
      </svg>
    ),
  },
  {
    label: "Stats",
    icon: (
      <svg {...stroke} stroke="currentColor">
        <path d="M4 20V4M4 20h16M9 16v-5M14 16V8M19 16v-3" />
      </svg>
    ),
  },
  {
    label: "Shield",
    icon: (
      <svg {...stroke} stroke="currentColor">
        <path d="M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6z" />
      </svg>
    ),
  },
  {
    label: "Trading desk",
    icon: (
      <svg {...stroke} stroke="currentColor">
        <path d="M7 4v16M17 4v16M5 8h4v6H5zM15 10h4v5h-4z" />
      </svg>
    ),
  },
];

export function Sidebar({ version }: { version: string | null }) {
  return (
    <nav
      aria-label="Main"
      data-tauri-drag-region
      className="flex w-[220px] shrink-0 flex-col gap-0.5 border-r border-hairline bg-white/[0.025] px-3 pt-11 pb-4"
    >
      <div className="flex items-center gap-2.5 px-2 pb-[18px]">
        <HaloMark size={26} />
        <span className="text-[19px] font-semibold tracking-[-0.03em]">halo</span>
      </div>
      <p className="mx-2.5 my-1.5 text-[11px] font-semibold tracking-[0.06em] text-text-muted uppercase">
        Workspace
      </p>
      {ITEMS.map((item) =>
        item.current ? (
          <a
            key={item.label}
            aria-current="page"
            className="flex items-center gap-2.5 rounded-lg bg-white/8 px-2.5 py-2 text-[13px] font-medium text-text"
          >
            {item.icon}
            {item.label}
          </a>
        ) : (
          <span
            key={item.label}
            aria-disabled="true"
            title="Coming in a later phase"
            className="flex items-center gap-2.5 rounded-lg px-2.5 py-2 text-[13px] font-medium text-text-secondary opacity-60"
          >
            {item.icon}
            {item.label}
          </span>
        ),
      )}
      <p className="mt-auto px-2.5 text-[11px] text-text-muted tabular-nums">
        {version ? `v${version}` : ""}
      </p>
    </nav>
  );
}
