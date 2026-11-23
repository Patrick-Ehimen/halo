import { DatePicker, Dropdown, TimePicker } from "@halo/ui";
import { useState } from "react";
import type { GoalForm, GoalView, MilestoneView, ProjectForm, ProjectView, TaskForm } from "../api";
import { ColorPicker, Dialog, Field, fieldClass, textareaClass } from "./ui";
import { nextColor } from "./values";

type Common = { onClose: () => void; error?: string | null };

export function GoalDialog({
  goal,
  usedColors = [],
  onSave,
  onDelete,
  onClose,
  error,
}: Common & {
  goal?: GoalView;
  /** Colours other goals have, so a new one gets its own. */
  usedColors?: string[];
  onSave: (form: GoalForm) => void;
  onDelete?: () => void;
}) {
  const [title, setTitle] = useState(goal?.title ?? "");
  const [why, setWhy] = useState(goal?.why ?? "");
  const [target, setTarget] = useState(String(goal?.targetSessions ?? 60));
  const [dueOn, setDueOn] = useState(goal?.dueOn ?? "");
  const [color, setColor] = useState(goal?.color ?? nextColor(usedColors));
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
        <Field label="By when" custom>
          <DatePicker label="By when" value={dueOn} onChange={setDueOn} placeholder="No deadline" />
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
  usedColors = [],
  onDelete,
  onClose,
  error,
}: Common & {
  project?: ProjectView;
  goals: GoalView[];
  defaultGoalId?: string | null;
  onSave: (form: ProjectForm) => void;
  usedColors?: string[];
  onDelete?: () => void;
}) {
  const [name, setName] = useState(project?.name ?? "");
  const [goalId, setGoalId] = useState(project?.goalId ?? defaultGoalId ?? "");
  const [color, setColor] = useState(project?.color ?? nextColor(usedColors));
  const [description, setDescription] = useState(project?.description ?? "");
  return (
    <Dialog
      title={project ? "Edit project" : "New project"}
      submitLabel={project ? "Save" : "Add project"}
      onClose={onClose}
      error={error}
      danger={onDelete && { label: "Delete project", onClick: onDelete }}
      onSubmit={() => onSave({ name, color, goalId: goalId || null, description })}
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
      <Field label="Description">
        <textarea
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="What this project is and what done looks like"
          rows={3}
          maxLength={2000}
          className={textareaClass}
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
  const [endTime, setEndTime] = useState(task?.endTime ?? "");
  const [description, setDescription] = useState(task?.description ?? "");
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
          // An end only means something after a start.
          endTime: (dueTime && endTime) || null,
          recurrence,
          description,
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
      <Field label="Description">
        <textarea
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Notes, links, what done looks like"
          rows={3}
          maxLength={2000}
          className={textareaClass}
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
      <div className="grid grid-cols-[80px_1fr] gap-3">
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
        <Field label="Day" custom>
          <DatePicker label="Day" value={plannedOn} onChange={setPlannedOn} placeholder="Someday" />
        </Field>
      </div>
      <div className="grid grid-cols-2 gap-3">
        <Field label="Starts" custom>
          <TimePicker label="Starts" value={dueTime} onChange={setDueTime} />
        </Field>
        <Field label="Ends" custom>
          <TimePicker
            label="Ends"
            value={endTime}
            onChange={setEndTime}
            placeholder={dueTime ? "--:--" : "Set a start"}
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

const LENGTHS = [25, 30, 50, 60, 90, 100, 120, 180];

function lengthLabel(m: number): string {
  if (m < 60) return `${m} min`;
  const h = Math.floor(m / 60);
  const rest = m % 60;
  return rest ? `${h} h ${rest} min` : `${h} h`;
}

/** Puts a task on today's timeline, or changes a block already on it. */
export function ScheduleDialog({
  title,
  editing = false,
  defaultStart,
  defaultMinutes,
  onSave,
  onDelete,
  onClose,
  error,
}: Common & {
  title: string;
  /** Changing a block already on the timeline, rather than adding one. */
  editing?: boolean;
  /** "HH:MM". */
  defaultStart: string;
  defaultMinutes: number;
  onSave: (start: string, minutes: number) => void;
  onDelete?: () => void;
}) {
  const [start, setStart] = useState(defaultStart);
  const [minutes, setMinutes] = useState(String(defaultMinutes));
  // Keep a block's own length on the list, even when it isn't a usual one.
  const lengths = LENGTHS.includes(defaultMinutes)
    ? LENGTHS
    : [...LENGTHS, defaultMinutes].sort((a, b) => a - b);
  return (
    <Dialog
      title={editing ? "Edit block" : "Block time"}
      submitLabel={editing ? "Save" : "Add to timeline"}
      onClose={onClose}
      error={error}
      danger={onDelete && { label: "Delete block", onClick: onDelete }}
      onSubmit={() => onSave(start, Number(minutes))}
    >
      <p className="text-[13px] text-text-secondary">{title}</p>
      <div className="grid grid-cols-2 gap-3">
        <Field label="Starts" custom>
          <TimePicker label="Starts" value={start} onChange={setStart} />
        </Field>
        <div className="grid gap-1.5 text-[13px] font-medium">
          Length
          <Dropdown
            label="Length"
            value={minutes}
            options={lengths.map((m) => ({ value: String(m), label: lengthLabel(m) }))}
            onChange={setMinutes}
          />
        </div>
      </div>
    </Dialog>
  );
}
