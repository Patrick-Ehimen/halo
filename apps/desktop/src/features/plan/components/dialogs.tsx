import { Dropdown } from "@halo/ui";
import { useState } from "react";
import type { GoalForm, GoalView, MilestoneView, ProjectForm, ProjectView, TaskForm } from "../api";
import { ColorPicker, Dialog, Field, fieldClass } from "./ui";
import { COLORS } from "./values";

type Common = { onClose: () => void; error?: string | null };

export function GoalDialog({
  goal,
  onSave,
  onDelete,
  onClose,
  error,
}: Common & {
  goal?: GoalView;
  onSave: (form: GoalForm) => void;
  onDelete?: () => void;
}) {
  const [title, setTitle] = useState(goal?.title ?? "");
  const [why, setWhy] = useState(goal?.why ?? "");
  const [target, setTarget] = useState(String(goal?.targetSessions ?? 60));
  const [dueOn, setDueOn] = useState(goal?.dueOn ?? "");
  const [color, setColor] = useState(goal?.color ?? COLORS[0] ?? "#F2B544");
  return (
    <Dialog
      title={goal ? "Edit goal" : "New goal"}
      submitLabel={goal ? "Save" : "Add goal"}
      onClose={onClose}
      error={error}
      danger={onDelete && { label: "Delete goal", onClick: onDelete }}
      onSubmit={() =>
        onSave({
          title,
          why,
          targetSessions: Number(target) || 0,
          dueOn: dueOn || null,
          color,
        })
      }
    >
      <Field label="What do you want to achieve?">
        <input
          autoFocus
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="Ship meridian-rs v1"
          className={fieldClass}
        />
      </Field>
      <Field label="Why it matters">
        <input
          value={why}
          onChange={(e) => setWhy(e.target.value)}
          placeholder="A fast, tested order book other people can build on"
          className={fieldClass}
        />
      </Field>
      <div className="grid grid-cols-2 gap-3">
        <Field label="Target sessions">
          <input
            type="number"
            min={1}
            max={10000}
            value={target}
            onChange={(e) => setTarget(e.target.value)}
            className={`${fieldClass} tabular-nums`}
          />
        </Field>
        <Field label="By when">
          <input
            type="date"
            value={dueOn}
            onChange={(e) => setDueOn(e.target.value)}
            className={fieldClass}
          />
        </Field>
      </div>
      {goal && goal.estimateTotal > 0 && (
        <p className="text-xs text-text-muted">
          Its tasks add up to {goal.estimateTotal} sessions.
        </p>
      )}
      <Field label="Colour">
        <ColorPicker value={color} onChange={setColor} />
      </Field>
    </Dialog>
  );
}

export function ProjectDialog({
  project,
  goals,
  defaultGoalId,
  onSave,
  onArchive,
  onClose,
  error,
}: Common & {
  project?: ProjectView;
  goals: GoalView[];
  defaultGoalId?: string | null;
  onSave: (form: ProjectForm) => void;
  onArchive?: () => void;
}) {
  const [name, setName] = useState(project?.name ?? "");
  const [goalId, setGoalId] = useState(project?.goalId ?? defaultGoalId ?? "");
  const [color, setColor] = useState(
    project?.color ?? goals.find((g) => g.id === defaultGoalId)?.color ?? COLORS[1] ?? "#6CB6FF",
  );
  return (
    <Dialog
      title={project ? "Edit project" : "New project"}
      submitLabel={project ? "Save" : "Add project"}
      onClose={onClose}
      error={error}
      danger={onArchive && { label: "Archive", onClick: onArchive }}
      onSubmit={() => onSave({ name, color, goalId: goalId || null })}
    >
      <Field label="Name">
        <input
          autoFocus
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="meridian-rs"
          className={fieldClass}
        />
      </Field>
      <div className="flex items-center justify-between gap-3 text-[13px] font-medium">
        Goal
        <Dropdown
          label="Goal"
          value={goalId}
          options={[
            { value: "", label: "No goal" },
            ...goals
              .filter((g) => g.status === "active")
              .map((g) => ({ value: g.id, label: g.title })),
          ]}
          onChange={setGoalId}
        />
      </div>
      <Field label="Colour">
        <ColorPicker value={color} onChange={setColor} />
      </Field>
    </Dialog>
  );
}

const WEEK = [
  ["MO", "Mon"],
  ["TU", "Tue"],
  ["WE", "Wed"],
  ["TH", "Thu"],
  ["FR", "Fri"],
  ["SA", "Sat"],
  ["SU", "Sun"],
] as const;

type RepeatKind = "none" | "daily" | "weekdays" | "weekly";

function repeatOf(r: string | null): { kind: RepeatKind; days: string[] } {
  if (r === "daily" || r === "weekdays") return { kind: r, days: [] };
  if (r?.startsWith("weekly:")) return { kind: "weekly", days: r.slice(7).split(",") };
  return { kind: "none", days: [] };
}

export function TaskDialog({
  task,
  projects,
  milestones,
  defaultDate,
  onSave,
  onDelete,
  onClose,
  error,
}: Common & {
  task?: TaskForm & { id?: string };
  projects: ProjectView[];
  milestones: MilestoneView[];
  defaultDate: string;
  onSave: (form: TaskForm) => void;
  onDelete?: () => void;
}) {
  const [title, setTitle] = useState(task?.title ?? "");
  const [projectId, setProjectId] = useState(task?.projectId ?? "");
  const [milestoneId, setMilestoneId] = useState(task?.milestoneId ?? "");
  const [estimate, setEstimate] = useState(String(task?.estimateSessions ?? 1));
  const [plannedOn, setPlannedOn] = useState(task?.plannedOn ?? defaultDate);
  const [dueTime, setDueTime] = useState(task?.dueTime ?? "");
  const initial = repeatOf(task?.recurrence ?? null);
  const [repeat, setRepeat] = useState<RepeatKind>(initial.kind);
  const [days, setDays] = useState<string[]>(initial.days);

  const goalId = projects.find((p) => p.id === projectId)?.goalId ?? null;
  const goalMilestones = milestones.filter((m) => m.goalId === goalId && !m.done);
  const recurrence =
    repeat === "none"
      ? null
      : repeat === "weekly"
        ? days.length
          ? `weekly:${WEEK.map(([c]) => c)
              .filter((c) => days.includes(c))
              .join(",")}`
          : null
        : repeat;

  return (
    <Dialog
      title={task?.id ? "Edit task" : "New task"}
      submitLabel={task?.id ? "Save" : "Add task"}
      onClose={onClose}
      error={error}
      danger={onDelete && { label: "Delete task", onClick: onDelete }}
      onSubmit={() =>
        onSave({
          title,
          projectId: projectId || null,
          milestoneId: goalMilestones.some((m) => m.id === milestoneId) ? milestoneId : null,
          parentId: task?.parentId ?? null,
          estimateSessions: Number(estimate) || 0,
          plannedOn: plannedOn || null,
          dueTime: dueTime || null,
          recurrence,
        })
      }
    >
      <Field label="Task">
        <input
          autoFocus
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="Write matching-engine tests"
          className={fieldClass}
        />
      </Field>
      <div className="flex items-center justify-between gap-3 text-[13px] font-medium">
        Project
        <Dropdown
          label="Project"
          value={projectId}
          options={[
            { value: "", label: "No project" },
            ...projects.map((p) => ({ value: p.id, label: p.name })),
          ]}
          onChange={setProjectId}
        />
      </div>
      {goalMilestones.length > 0 && (
        <div className="flex items-center justify-between gap-3 text-[13px] font-medium">
          Milestone
          <Dropdown
            label="Milestone"
            value={milestoneId}
            options={[
              { value: "", label: "None" },
              ...goalMilestones.map((m) => ({ value: m.id, label: m.title })),
            ]}
            onChange={setMilestoneId}
          />
        </div>
      )}
      <div className="grid grid-cols-[80px_1fr_120px] gap-3">
        <Field label="Sessions">
          <input
            type="number"
            min={0}
            max={50}
            value={estimate}
            onChange={(e) => setEstimate(e.target.value)}
            className={`${fieldClass} tabular-nums`}
          />
        </Field>
        <Field label="Day">
          <input
            type="date"
            value={plannedOn}
            onChange={(e) => setPlannedOn(e.target.value)}
            className={fieldClass}
          />
        </Field>
        <Field label="Time">
          <input
            type="time"
            value={dueTime}
            onChange={(e) => setDueTime(e.target.value)}
            className={fieldClass}
          />
        </Field>
      </div>
      <div className="flex items-center justify-between gap-3 text-[13px] font-medium">
        Repeat
        <Dropdown
          label="Repeat"
          value={repeat}
          options={[
            { value: "none", label: "Doesn’t repeat" },
            { value: "daily", label: "Daily" },
            { value: "weekdays", label: "Weekdays" },
            { value: "weekly", label: "Weekly on…" },
          ]}
          onChange={(v) => setRepeat(v)}
        />
      </div>
      {repeat === "weekly" && (
        <div role="group" aria-label="Repeat on" className="flex gap-1.5">
          {WEEK.map(([code, name]) => {
            const on = days.includes(code);
            return (
              <button
                key={code}
                type="button"
                aria-pressed={on}
                onClick={() => setDays((d) => (on ? d.filter((x) => x !== code) : [...d, code]))}
                className={`h-11 flex-1 rounded-control text-[13px] font-medium ${
                  on
                    ? "bg-focus text-focus-ink"
                    : "border border-white/10 bg-control text-text-secondary"
                }`}
              >
                {name}
              </button>
            );
          })}
        </div>
      )}
    </Dialog>
  );
}

/** Puts a task on today's timeline. */
export function ScheduleDialog({
  title,
  defaultStart,
  defaultMinutes,
  onSave,
  onClose,
  error,
}: Common & {
  title: string;
  /** "HH:MM". */
  defaultStart: string;
  defaultMinutes: number;
  onSave: (start: string, minutes: number) => void;
}) {
  const [start, setStart] = useState(defaultStart);
  const [minutes, setMinutes] = useState(String(defaultMinutes));
  return (
    <Dialog
      title="Block time"
      submitLabel="Add to timeline"
      onClose={onClose}
      error={error}
      onSubmit={() => onSave(start, Number(minutes))}
    >
      <p className="text-[13px] text-text-secondary">{title}</p>
      <div className="grid grid-cols-2 gap-3">
        <Field label="Starts">
          <input
            type="time"
            value={start}
            onChange={(e) => setStart(e.target.value)}
            className={fieldClass}
          />
        </Field>
        <div className="grid gap-1.5 text-[13px] font-medium">
          Length
          <Dropdown
            label="Length"
            value={minutes}
            options={["25", "30", "50", "60", "90", "100", "120", "180"].map((m) => ({
              value: m,
              label: Number(m) >= 60 && Number(m) % 60 === 0 ? `${Number(m) / 60} h` : `${m} min`,
            }))}
            onChange={setMinutes}
          />
        </div>
      </div>
    </Dialog>
  );
}
