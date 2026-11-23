import type { TaskView } from "../api";
import { Dot, SessionDots } from "./ui";

type TaskRowProps = {
  task: TaskView;
  color: string;
  projectName?: string;
  milestoneTitle?: string;
  subtasks?: TaskView[];
  /** Hide the start action, e.g. while a session runs. */
  canStart: boolean;
  onToggle: (task: TaskView) => void;
  onStart: (task: TaskView) => void;
  onEdit: (task: TaskView) => void;
  onSchedule?: (task: TaskView) => void;
};

const FlagIcon = (
  <svg
    width="11"
    height="11"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.2"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
  >
    <path d="M5 21V4" />
    <path d="M5 4h11l-2 4 2 4H5" />
  </svg>
);

function Check({ task, color, onToggle }: { task: TaskView; color: string; onToggle: () => void }) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={task.completed}
      aria-label={task.completed ? `Reopen ${task.title}` : `Complete ${task.title}`}
      onClick={onToggle}
      className="-m-3 flex size-11 shrink-0 items-center justify-center"
    >
      <span
        className="flex size-[18px] items-center justify-center rounded-[5px]"
        style={
          task.completed
            ? { background: color }
            : { boxSizing: "border-box", border: "1.5px solid rgba(255,255,255,0.28)" }
        }
      >
        {task.completed && (
          <svg
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="#2A1D05"
            strokeWidth="3"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <path d="M5 12.5l4.5 4.5L19 7.5" />
          </svg>
        )}
      </span>
    </button>
  );
}

/** One task in the plan: tick it off, start a session on it, or open it. */
export function TaskRow({
  task,
  color,
  projectName,
  milestoneTitle,
  subtasks = [],
  canStart,
  onToggle,
  onStart,
  onEdit,
  onSchedule,
}: TaskRowProps) {
  const subDone = subtasks.filter((s) => s.completed).length;
  const meta = [
    task.whenLabel,
    projectName,
    subtasks.length ? `${subDone} of ${subtasks.length} subtasks` : null,
    task.recurrenceLabel,
  ].filter(Boolean);

  return (
    <div className="group rounded-card border border-card-border bg-card">
      <div className="flex items-center gap-3 py-2.5 pr-2.5 pl-3.5">
        <Check task={task} color={color} onToggle={() => onToggle(task)} />
        <button
          type="button"
          onClick={() => onEdit(task)}
          className="min-w-0 flex-1 text-left"
          aria-label={`Edit ${task.title}`}
        >
          <span
            className={`block truncate text-sm font-medium ${
              task.completed ? "text-text-muted line-through" : "text-text"
            }`}
          >
            {task.title}
          </span>
          {task.description && (
            <span className="block truncate text-xs text-text-secondary">{task.description}</span>
          )}
          <span className="flex items-center gap-1 truncate text-xs text-text-muted">
            {milestoneTitle && (
              <span className="inline-flex shrink-0 items-center gap-1" style={{ color }}>
                {FlagIcon}
                {milestoneTitle}
                {meta.length > 0 && <span className="text-text-muted"> ·</span>}
              </span>
            )}
            <span className={task.overdue ? "text-alert" : ""}>{meta.join(" · ")}</span>
          </span>
        </button>
        <SessionDots done={task.doneSessions} total={task.estimateSessions} color={color} />
        {!task.completed && (
          <span className="flex gap-1 opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100">
            {onSchedule && (
              <button
                type="button"
                aria-label={`Block time for ${task.title}`}
                onClick={() => onSchedule(task)}
                className="flex size-9 items-center justify-center rounded-full text-text-secondary hover:bg-white/8 hover:text-text"
              >
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.8"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  aria-hidden="true"
                >
                  <rect x="3" y="4" width="18" height="17" rx="2" />
                  <path d="M16 2v4M8 2v4M3 10h18" />
                </svg>
              </button>
            )}
            {canStart && (
              <button
                type="button"
                aria-label={`Start ${task.title}`}
                onClick={() => onStart(task)}
                className="flex size-9 items-center justify-center rounded-full bg-focus/15 text-focus hover:bg-focus/25"
              >
                <svg
                  width="13"
                  height="13"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                  aria-hidden="true"
                >
                  <path d="M8 5l12 7-12 7z" />
                </svg>
              </button>
            )}
          </span>
        )}
      </div>
      {subtasks.length > 0 && (
        <ul className="border-t border-hairline py-1 pr-3 pl-11">
          {subtasks.map((s) => (
            <li key={s.id} className="flex items-center gap-3 py-1">
              <Check task={s} color={color} onToggle={() => onToggle(s)} />
              <button
                type="button"
                onClick={() => onEdit(s)}
                className={`min-w-0 flex-1 truncate text-left text-[13px] ${
                  s.completed ? "text-text-muted line-through" : "text-text-secondary"
                }`}
              >
                {s.title}
              </button>
              <Dot color={s.completed ? color : "rgba(255,255,255,0.2)"} />
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
