import { useEffect, useState } from "react";
import { useFocusStore } from "../focus/store";
import { presetOf } from "../focus/format";
import { planApi, type TaskForm, type TaskView } from "./api";
import { ScheduleDialog, TaskDialog } from "./components/dialogs";
import { QuickAdd } from "./components/QuickAdd";
import { TaskRow } from "./components/TaskRow";
import { Timeline } from "./components/Timeline";
import { GoalRing, PaceBadge } from "./components/ui";
import { formOf, model, sessions } from "./model";
import { usePlanStore } from "./store";

type PlanScreenProps = { onOpenGoals: (goalId?: string) => void };

/** Today plan: tasks grouped by the goals they move, and the day's timeline. */
export function PlanScreen({ onOpenGoals }: PlanScreenProps) {
  const { view, error, connect, run, clearError } = usePlanStore();
  const timer = useFocusStore((s) => s.snapshot);
  const connectTimer = useFocusStore((s) => s.connect);
  const [editing, setEditing] = useState<(TaskForm & { id?: string }) | null>(null);
  const [scheduling, setScheduling] = useState<TaskView | null>(null);

  useEffect(() => connect(), [connect]);
  useEffect(() => connectTimer(), [connectTimer]);
  useEffect(() => {
    if (!error || editing || scheduling) return;
    const t = setTimeout(clearError, 4000);
    return () => clearTimeout(t);
  }, [error, editing, scheduling, clearError]);

  if (!view) {
    return (
      <div className="flex flex-1 items-center justify-center text-sm text-text-muted">
        {error ? `Plan unavailable: ${error}` : "Loading your plan"}
      </div>
    );
  }

  const m = model(view);
  const groups = m.groups();
  const canStart = timer?.status === "idle" && timer.phase === "focus";
  const activeGoals = view.goals.filter((g) => g.status === "active");
  const plannedToday = view.today
    .filter((t) => !t.parentId && !t.completed)
    .reduce((n, t) => n + Math.max(0, t.estimateSessions - t.doneSessions), 0);
  const goalsToday = activeGoals.filter((g) => g.todaySessions > 0).length;
  const blockedTasks = new Set(view.blocks.map((b) => b.taskId));
  const timed = view.today.filter(
    (t) => t.dueMinute !== null && !t.completed && !blockedTasks.has(t.id),
  );

  const save = async (form: TaskForm, id?: string) => {
    const ok = await run(() => (id ? planApi.updateTask(id, form) : planApi.createTask(form)));
    if (ok) setEditing(null);
  };
  const start = (t: TaskView) =>
    void run(() => planApi.startTask(t.id, timer ? presetOf(timer) : "pomodoro"));

  return (
    <>
      <header
        data-tauri-drag-region
        className="flex flex-wrap items-center gap-3 border-b border-hairline px-7 py-4"
      >
        <div data-tauri-drag-region className="flex-1">
          <h1 className="text-lg font-semibold tracking-[-0.01em]">Today plan</h1>
          <p className="text-[13px] text-text-secondary">
            {view.dateLabel} · {sessions(plannedToday)} planned
            {goalsToday > 0 && ` toward ${goalsToday} ${goalsToday === 1 ? "goal" : "goals"}`}
          </p>
        </div>
        <button
          type="button"
          onClick={() =>
            setEditing({
              title: "",
              projectId: null,
              milestoneId: null,
              parentId: null,
              estimateSessions: 1,
              plannedOn: view.date,
              dueTime: null,
              recurrence: null,
            })
          }
          className="h-11 rounded-control border border-white/10 bg-control px-4 text-[13px] font-medium text-text hover:bg-white/9"
        >
          New task
        </button>
      </header>

      <div className="flex min-h-0 flex-1 flex-col gap-[18px] overflow-auto px-7 pt-5 pb-6">
        <section aria-label="Today moves your goals" className="flex flex-col gap-2.5">
          <div className="flex items-baseline justify-between">
            <p className="text-[11px] font-semibold tracking-[0.06em] text-text-muted uppercase">
              Today moves your goals
            </p>
            <button
              type="button"
              onClick={() => onOpenGoals()}
              className="min-h-9 text-xs font-medium text-focus"
            >
              All goals
            </button>
          </div>
          {activeGoals.length === 0 ? (
            <button
              type="button"
              onClick={() => onOpenGoals()}
              className="rounded-card border border-dashed border-white/14 p-4 text-[13px] text-text-secondary hover:bg-white/4"
            >
              Set a goal, so today’s sessions add up to something
            </button>
          ) : (
            <div className="flex flex-wrap gap-3">
              {activeGoals.map((g) => (
                <button
                  key={g.id}
                  type="button"
                  onClick={() => onOpenGoals(g.id)}
                  className="flex min-w-[200px] flex-1 items-center gap-3 rounded-card border border-card-border bg-card px-3.5 py-3 text-left hover:bg-white/6"
                >
                  <GoalRing
                    size={44}
                    stroke={5}
                    percent={g.percent}
                    color={g.color}
                    planned={g.targetSessions ? (g.todaySessions * 100) / g.targetSessions : 0}
                    label={`${g.percent}%`}
                  />
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-[13px] font-semibold">{g.title}</span>
                    <span className="mt-0.5 flex items-center gap-1.5 text-xs">
                      <span className="font-medium" style={{ color: g.color }}>
                        +{g.todaySessions} today
                      </span>
                      <PaceBadge pace={g.pace} />
                    </span>
                  </span>
                </button>
              ))}
            </div>
          )}
        </section>

        <QuickAdd preview={planApi.preview} onAdd={(text) => run(() => planApi.quickAdd(text))} />
        {error && !editing && !scheduling && (
          <p role="status" className="-mt-2 text-[13px] text-alert">
            {error}
          </p>
        )}

        <div className="flex flex-wrap gap-[18px]">
          <section
            aria-label="Tasks by goal"
            className="flex min-w-0 flex-[1_1_380px] flex-col gap-4"
          >
            {groups.length === 0 && (
              <p className="rounded-card border border-card-border bg-card p-5 text-[13px] text-text-secondary">
                Nothing planned yet. Type a task above, like “write tests ~2 sessions #project”.
              </p>
            )}
            {groups.map(({ goal, tasks }) => (
              <div key={goal?.id ?? "none"} className="flex flex-col gap-2">
                <div className="flex items-center gap-2 px-0.5">
                  {goal ? (
                    <GoalRing size={16} stroke={2.6} percent={goal.percent} color={goal.color} />
                  ) : (
                    <span
                      className="size-4 rounded-full border-2 border-white/14"
                      aria-hidden="true"
                    />
                  )}
                  <h2 className="text-sm font-semibold">{goal?.title ?? "No goal"}</h2>
                  <span className="ml-auto text-xs text-text-muted">
                    {sessions(
                      tasks
                        .filter((t) => !t.completed)
                        .reduce((n, t) => n + Math.max(0, t.estimateSessions - t.doneSessions), 0),
                    )}
                  </span>
                </div>
                {tasks.map((t) => (
                  <TaskRow
                    key={t.id}
                    task={t}
                    color={m.colorOf(t)}
                    projectName={m.projectOf(t)?.name}
                    milestoneTitle={
                      t.milestoneId ? m.milestones.get(t.milestoneId)?.title : undefined
                    }
                    subtasks={m.subtasksOf(t)}
                    canStart={canStart}
                    onToggle={(x) =>
                      void run(() =>
                        x.completed ? planApi.reopenTask(x.id) : planApi.completeTask(x.id),
                      )
                    }
                    onStart={start}
                    onEdit={(x) => setEditing(formOf(x))}
                    onSchedule={setScheduling}
                  />
                ))}
              </div>
            ))}
          </section>

          <div className="min-w-0 flex-[1_1_340px]">
            <Timeline
              blocks={view.blocks}
              timedTasks={timed}
              nowMinute={view.nowMinute}
              colorOf={(taskId) => {
                const t = taskId ? m.allTasks.get(taskId) : undefined;
                return t ? m.colorOf(t) : null;
              }}
              onDeleteBlock={(id) => void run(() => planApi.deleteBlock(id))}
            />
          </div>
        </div>
      </div>

      {editing && (
        <TaskDialog
          task={editing}
          projects={view.projects}
          milestones={view.goals.flatMap((g) => g.milestones)}
          defaultDate={view.date}
          error={error}
          onClose={() => {
            setEditing(null);
            clearError();
          }}
          onSave={(form) => void save(form, editing.id)}
          onDelete={
            editing.id
              ? () =>
                  void run(() => planApi.deleteTask(editing.id ?? "")).then(
                    (ok) => ok && setEditing(null),
                  )
              : undefined
          }
        />
      )}
      {scheduling && (
        <ScheduleDialog
          title={scheduling.title}
          defaultStart={scheduling.dueTime ?? "09:00"}
          defaultMinutes={50}
          error={error}
          onClose={() => {
            setScheduling(null);
            clearError();
          }}
          onSave={(start, minutes) =>
            void run(() =>
              planApi.createBlock(scheduling.id, scheduling.title, start, minutes),
            ).then((ok) => ok && setScheduling(null))
          }
        />
      )}
    </>
  );
}
