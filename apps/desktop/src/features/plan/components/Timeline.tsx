import type { BlockView, TaskView } from "../api";

type Item = {
  id: string;
  title: string;
  meta: string;
  start: number;
  end: number;
  color: string | null;
  /** Blocks can be removed; timed tasks are shown from their due time. */
  blockId?: string;
};

type TimelineProps = {
  blocks: BlockView[];
  /** Today's timed tasks without a block, shown at their due time. */
  timedTasks: TaskView[];
  nowMinute: number;
  colorOf: (taskId: string | null) => string | null;
  onDeleteBlock: (id: string) => void;
};

const HOUR_PX = 56;

/** The day's time blocks, coloured by goal, with a line for now. */
export function Timeline({ blocks, timedTasks, nowMinute, colorOf, onDeleteBlock }: TimelineProps) {
  const items: Item[] = [
    ...blocks.map((b) => ({
      id: `b-${b.id}`,
      blockId: b.id,
      title: b.title,
      meta: b.label,
      start: b.startMinute,
      end: b.endMinute,
      color: colorOf(b.taskId),
    })),
    ...timedTasks.map((t) => ({
      id: `t-${t.id}`,
      title: t.title,
      meta: `${t.whenLabel ?? ""} · ${t.estimateSessions} ${t.estimateSessions === 1 ? "session" : "sessions"}`,
      start: t.dueMinute ?? 0,
      end: t.dueEndMinute ?? t.dueMinute ?? 0,
      color: colorOf(t.id),
    })),
  ].sort((a, b) => a.start - b.start);

  // Show 08:00–19:00, widened to fit everything on the day and the now line.
  const first = Math.min(
    8,
    ...items.map((i) => Math.floor(i.start / 60)),
    Math.floor(nowMinute / 60),
  );
  const last = Math.max(19, ...items.map((i) => Math.ceil(i.end / 60)), Math.ceil(nowMinute / 60));
  const top = (minute: number) => ((minute - first * 60) / 60) * HOUR_PX;
  const hours = Array.from({ length: last - first + 1 }, (_, i) => first + i);

  return (
    <section
      aria-label="Time blocks"
      className="rounded-card border border-card-border bg-card p-4"
    >
      <div className="mb-3 flex items-baseline justify-between">
        <h2 className="text-sm font-semibold">Time blocks</h2>
        <span className="text-xs text-text-muted">Coloured by goal</span>
      </div>
      <div className="relative" style={{ height: (last - first) * HOUR_PX }}>
        {hours.map((h) => (
          <div
            key={h}
            className="absolute inset-x-0 flex items-center gap-2.5"
            style={{ top: (h - first) * HOUR_PX - 7 }}
          >
            <span className="w-10 text-[11px] text-text-muted tabular-nums">
              {String(h % 24).padStart(2, "0")}:00
            </span>
            <span className="h-px flex-1 bg-white/5" />
          </div>
        ))}
        {items.map((it) => {
          const height = Math.max(24, top(it.end) - top(it.start) - 4);
          const style = it.color
            ? { background: `${it.color}1F`, border: `1px solid ${it.color}55` }
            : { background: "rgba(255,255,255,0.04)", border: "1px solid rgba(255,255,255,0.08)" };
          return (
            <div
              key={it.id}
              className="group absolute right-1 left-[52px] overflow-hidden rounded-control px-2.5 py-1.5"
              style={{ top: top(it.start) + 2, height, ...style }}
            >
              <div
                className="truncate text-[13px] font-semibold"
                style={{ color: it.color ?? "var(--halo-text-secondary)" }}
              >
                {it.title}
              </div>
              {height > 34 && (
                <div className="truncate text-[11px] text-text-secondary">{it.meta}</div>
              )}
              {it.blockId && (
                <button
                  type="button"
                  aria-label={`Remove block ${it.title}`}
                  onClick={() => onDeleteBlock(it.blockId ?? "")}
                  className="absolute top-1 right-1 flex size-7 items-center justify-center rounded-full text-text-muted opacity-0 group-hover:opacity-100 hover:bg-white/10 hover:text-text"
                >
                  <svg
                    width="10"
                    height="10"
                    viewBox="0 0 12 12"
                    stroke="currentColor"
                    strokeWidth="1.6"
                    strokeLinecap="round"
                    aria-hidden="true"
                  >
                    <path d="M3 3l6 6M9 3l-6 6" />
                  </svg>
                </button>
              )}
            </div>
          );
        })}
        <div
          className="pointer-events-none absolute right-0 left-12 h-0.5 bg-focus"
          style={{ top: top(nowMinute) }}
        >
          <span className="absolute -top-2.5 right-1 rounded-md bg-focus px-1.5 py-px text-[11px] font-semibold text-focus-ink tabular-nums">
            Now
          </span>
        </div>
      </div>
    </section>
  );
}
