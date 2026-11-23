import { useEffect, useState } from "react";
import { presetOf } from "../focus/format";
import { useFocusStore } from "../focus/store";
import { planApi, type GoalView, type ProjectView } from "./api";
import { GoalDialog, ProjectDialog } from "./components/dialogs";
import { Dot, GoalRing, LinkButton, PaceBadge, fieldClass } from "./components/ui";
import { paceLabel } from "./components/values";
import { model, sessions } from "./model";
import { usePlanStore } from "./store";

type GoalsScreenProps = {
  selectedId: string | null;
  onSelect: (goalId: string) => void;
  onOpenPlan: () => void;
};

/** "4.5", "7". Display only: Rust sends tenths. */
function perWeek(tenths: number): string {
  return tenths % 10 === 0 ? String(tenths / 10) : (tenths / 10).toFixed(1);
}

/** Goals: what today's sessions are for. */
export function GoalsScreen({ selectedId, onSelect, onOpenPlan }: GoalsScreenProps) {
  const { view, error, connect, run, clearError } = usePlanStore();
  const timer = useFocusStore((s) => s.snapshot);
  const connectTimer = useFocusStore((s) => s.connect);
  const [tab, setTab] = useState<"active" | "done">("active");
  const [goalDialog, setGoalDialog] = useState<"new" | GoalView | null>(null);
  const [projectDialog, setProjectDialog] = useState<"new" | ProjectView | null>(null);
  const [milestone, setMilestone] = useState({ title: "", dueOn: "" });

  useEffect(() => connect(), [connect]);
  useEffect(() => connectTimer(), [connectTimer]);

  if (!view) {
    return (
      <div className="flex flex-1 items-center justify-center text-sm text-text-muted">
        {error ? `Goals unavailable: ${error}` : "Loading your goals"}
      </div>
    );
  }

  const m = model(view);
  const shown = view.goals.filter((g) => g.status === tab);
  const goal = view.goals.find((g) => g.id === selectedId) ?? shown[0] ?? null;
  const goalProjects = goal ? view.projects.filter((p) => p.goalId === goal.id) : [];
  const todayTasks = goal
    ? view.today.filter((t) => !t.parentId && !t.completed && m.goalOf(t)?.id === goal.id)
    : [];
  const canStart = timer?.status === "idle" && timer.phase === "focus";
  const doneMilestones = goal?.milestones.filter((x) => x.done).length ?? 0;
  const closeDialogs = () => {
    setGoalDialog(null);
    setProjectDialog(null);
    clearError();
  };

  return (
    <>
      <header
        data-tauri-drag-region
        className="flex flex-wrap items-center gap-3 border-b border-hairline px-7 py-4"
      >
        <div data-tauri-drag-region className="flex-1">
          <h1 className="text-lg font-semibold tracking-[-0.01em]">Goals</h1>
          <p className="text-[13px] text-text-secondary">
            {view.goals.filter((g) => g.status === "active").length} active · what today’s sessions
            are for
          </p>
        </div>
        <div
          role="group"
          aria-label="Show goals"
          className="flex gap-0.5 rounded-control border border-white/7 bg-control p-[3px]"
        >
          {(["active", "done"] as const).map((t) => (
            <button
              key={t}
              type="button"
              aria-pressed={tab === t}
              onClick={() => setTab(t)}
              className={`h-9 rounded-lg px-3.5 text-[13px] font-medium ${
                tab === t ? "bg-control-selected text-text" : "text-text-secondary hover:text-text"
              }`}
            >
              {t === "active" ? "Active" : "Done"}
            </button>
          ))}
        </div>
        <button
          type="button"
          onClick={() => setGoalDialog("new")}
          className="h-11 rounded-control bg-focus px-4 text-[13px] font-semibold text-focus-ink"
        >
          New goal
        </button>
      </header>

      <div className="flex min-h-0 flex-1 flex-wrap overflow-auto">
        <section
          aria-label="Goal list"
          className="flex w-[380px] shrink-0 flex-col gap-3 border-r border-hairline py-6 pr-5 pl-7"
        >
          {shown.map((g) => {
            const on = goal?.id === g.id;
            const projects = view.projects.filter((p) => p.goalId === g.id);
            return (
              <button
                key={g.id}
                type="button"
                aria-pressed={on}
                onClick={() => onSelect(g.id)}
                className="flex flex-col gap-3 rounded-card border p-4 text-left transition-colors"
                style={
                  on
                    ? { borderColor: `${g.color}66`, background: `${g.color}0F` }
                    : {
                        borderColor: "rgba(255,255,255,0.08)",
                        background: "rgba(255,255,255,0.045)",
                      }
                }
              >
                <span className="flex items-center gap-3.5">
                  <GoalRing
                    size={52}
                    stroke={6}
                    percent={g.percent}
                    color={g.color}
                    label={`${g.percent}%`}
                  />
                  <span className="min-w-0 flex-1">
                    <span className="block text-[15px] font-semibold">{g.title}</span>
                    {g.why && (
                      <span className="block truncate text-xs text-text-secondary">{g.why}</span>
                    )}
                  </span>
                  <PaceBadge pace={g.pace} />
                </span>
                {projects.length > 0 && (
                  <span className="flex flex-wrap gap-1.5">
                    {projects.map((p) => (
                      <span
                        key={p.id}
                        className="flex items-center gap-1.5 rounded-full border border-white/7 bg-white/5 px-2.5 py-[3px] text-xs text-text-secondary"
                      >
                        <Dot color={p.color} />
                        {p.name}
                      </span>
                    ))}
                  </span>
                )}
                <span className="flex items-center gap-2.5 text-xs text-text-muted">
                  <span className="whitespace-nowrap">
                    {g.dueLabel ? `By ${g.dueLabel} · ${g.timeLeft}` : "No date"}
                  </span>
                  <span className="ml-auto whitespace-nowrap text-text-secondary tabular-nums">
                    {g.thisWeek} this week
                  </span>
                </span>
              </button>
            );
          })}
          {tab === "active" && (
            <button
              type="button"
              onClick={() => setGoalDialog("new")}
              className="rounded-card border border-dashed border-white/14 p-4 text-[13px] text-text-secondary hover:bg-white/4"
            >
              Set a new goal: what, why, and by when
            </button>
          )}
          {tab === "done" && shown.length === 0 && (
            <p className="text-[13px] text-text-muted">Goals you finish show up here.</p>
          )}
        </section>

        {goal ? (
          <section
            aria-label={goal.title}
            className="flex min-w-0 flex-[999_1_440px] flex-col gap-4 px-7 py-6"
          >
            <div className="flex items-start gap-4">
              <GoalRing
                size={72}
                stroke={8}
                percent={goal.percent}
                color={goal.color}
                planned={goal.targetSessions ? (goal.todaySessions * 100) / goal.targetSessions : 0}
                label={`${goal.percent}%`}
              />
              <div className="min-w-0 flex-1">
                <p className="text-[11px] font-semibold tracking-[0.06em] text-text-muted uppercase">
                  Goal{goal.dueLabel ? ` · by ${goal.dueLabel}` : ""}
                </p>
                <h2 className="mt-1 text-2xl font-semibold tracking-[-0.02em]">{goal.title}</h2>
                {goal.why && <p className="text-sm text-text-secondary">{goal.why}</p>}
              </div>
              <button
                type="button"
                onClick={() => void run(() => planApi.setGoalDone(goal.id, goal.status !== "done"))}
                className="h-11 rounded-control border border-white/10 bg-control px-3.5 text-[13px] font-medium"
              >
                {goal.status === "done" ? "Make active" : "Mark done"}
              </button>
              <button
                type="button"
                onClick={() => setGoalDialog(goal)}
                className="h-11 rounded-control border border-white/10 bg-control px-3.5 text-[13px] font-medium"
              >
                Edit
              </button>
            </div>

            <div className="flex flex-wrap gap-3">
              {[
                [
                  "Progress",
                  `${goal.doneSessions} of ${goal.targetSessions}`,
                  `sessions · ${goal.percent}%`,
                ],
                [
                  "Pace",
                  `${perWeek(goal.averagedPerWeekTenths)} a week`,
                  goal.neededPerWeekTenths !== null
                    ? `${perWeek(goal.neededPerWeekTenths)} needed to finish on time`
                    : paceLabel(goal.pace),
                ],
                [
                  "Deadline",
                  goal.timeLeft ?? "No date",
                  goal.dueLabel ?? "Add a date to see your pace",
                ],
              ].map(([label, value, sub]) => (
                <div
                  key={label}
                  className="min-w-[150px] flex-1 rounded-card border border-card-border bg-card p-3.5"
                >
                  <p className="text-[11px] font-semibold tracking-[0.06em] text-text-muted uppercase">
                    {label}
                  </p>
                  <p className="mt-1.5 text-[22px] font-light tracking-[-0.01em] tabular-nums">
                    {value}
                  </p>
                  <p className="text-xs text-text-muted">{sub}</p>
                </div>
              ))}
            </div>

            <div className="flex flex-wrap gap-4">
              <div className="min-w-[280px] flex-1 rounded-card border border-card-border bg-card p-4">
                <div className="mb-3 flex items-center justify-between">
                  <h3 className="text-sm font-semibold">Milestones</h3>
                  <span className="text-xs text-text-muted">
                    {doneMilestones} of {goal.milestones.length} done
                  </span>
                </div>
                <ul className="flex flex-col gap-1">
                  {goal.milestones.map((ms) => (
                    <li key={ms.id} className="group flex items-center gap-3">
                      <button
                        type="button"
                        role="checkbox"
                        aria-checked={ms.done}
                        aria-label={ms.done ? `Reopen ${ms.title}` : `Mark ${ms.title} done`}
                        onClick={() => void run(() => planApi.setMilestoneDone(ms.id, !ms.done))}
                        className="flex size-11 shrink-0 items-center justify-center"
                      >
                        <span
                          className="flex size-5 items-center justify-center rounded-full"
                          style={
                            ms.done
                              ? { background: goal.color }
                              : {
                                  boxSizing: "border-box",
                                  border: "2px solid rgba(255,255,255,0.18)",
                                }
                          }
                        >
                          {ms.done && (
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
                      <span className="min-w-0 flex-1">
                        <span
                          className={`block text-sm font-medium ${ms.done ? "text-text-secondary" : ""}`}
                        >
                          {ms.title}
                        </span>
                        <span className="block text-xs text-text-muted">{ms.label}</span>
                      </span>
                      <button
                        type="button"
                        aria-label={`Delete ${ms.title}`}
                        onClick={() => void run(() => planApi.deleteMilestone(ms.id))}
                        className="flex size-9 items-center justify-center rounded-full text-text-muted opacity-0 group-hover:opacity-100 hover:bg-white/8 hover:text-text focus-visible:opacity-100"
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
                    </li>
                  ))}
                </ul>
                <form
                  className="mt-2 grid grid-cols-[1fr_auto] gap-2"
                  onSubmit={(e) => {
                    e.preventDefault();
                    void run(() =>
                      planApi.createMilestone(goal.id, milestone.title, milestone.dueOn || null),
                    ).then((ok) => ok && setMilestone({ title: "", dueOn: "" }));
                  }}
                >
                  <input
                    aria-label="New milestone"
                    value={milestone.title}
                    onChange={(e) => setMilestone({ ...milestone, title: e.target.value })}
                    placeholder="Add a milestone"
                    className={`${fieldClass} col-span-2`}
                  />
                  <input
                    aria-label="Milestone due date"
                    type="date"
                    value={milestone.dueOn}
                    onChange={(e) => setMilestone({ ...milestone, dueOn: e.target.value })}
                    className={fieldClass}
                  />
                  <button
                    type="submit"
                    disabled={!milestone.title.trim()}
                    className="h-11 rounded-control border border-white/10 bg-control px-4 text-[13px] font-medium text-text hover:bg-white/9 disabled:opacity-40"
                  >
                    Add
                  </button>
                </form>
              </div>

              <div className="min-w-[280px] flex-1 rounded-card border border-focus/22 bg-focus/7 p-4">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-semibold">Today for this goal</h3>
                  <span className="rounded-full bg-focus/12 px-2.5 py-[3px] text-xs font-medium text-focus">
                    +{sessions(goal.todaySessions)}
                  </span>
                </div>
                <p className="mt-1 mb-1.5 text-xs text-text-secondary">
                  {todayTasks.length
                    ? "From your Today plan."
                    : "Nothing planned for this goal today."}
                </p>
                {todayTasks.map((t) => (
                  <div
                    key={t.id}
                    className="flex items-center gap-2.5 border-t border-hairline py-2.5"
                  >
                    <Dot color={m.colorOf(t)} />
                    <span className="min-w-0 flex-1">
                      <span className="block truncate text-[13px] font-medium">{t.title}</span>
                      <span className="block truncate text-xs text-text-muted">
                        {[
                          m.projectOf(t)?.name,
                          t.milestoneId ? m.milestones.get(t.milestoneId)?.title : null,
                        ]
                          .filter(Boolean)
                          .join(" · ")}
                      </span>
                    </span>
                    <span className="text-xs text-text-secondary tabular-nums">
                      {sessions(Math.max(0, t.estimateSessions - t.doneSessions))}
                    </span>
                    {canStart && (
                      <button
                        type="button"
                        aria-label={`Start ${t.title}`}
                        onClick={() =>
                          void run(() =>
                            planApi.startTask(t.id, timer ? presetOf(timer) : "pomodoro"),
                          )
                        }
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
                  </div>
                ))}
                <LinkButton onClick={onOpenPlan}>Open today plan</LinkButton>
              </div>
            </div>

            <div className="rounded-card border border-card-border bg-card px-4 pb-1.5">
              <div className="flex items-center justify-between pt-3 pb-1">
                <h3 className="text-sm font-semibold">Projects</h3>
                <LinkButton onClick={() => setProjectDialog("new")}>Add project</LinkButton>
              </div>
              {goalProjects.length === 0 && (
                <p className="border-t border-hairline py-3 text-[13px] text-text-muted">
                  Add a project, so its tasks count toward this goal.
                </p>
              )}
              {goalProjects.map((p) => (
                <button
                  key={p.id}
                  type="button"
                  onClick={() => setProjectDialog(p)}
                  className="flex w-full items-center gap-3 border-t border-hairline py-2.5 text-left"
                >
                  <Dot color={p.color} size={8} />
                  <span className="w-[160px]">
                    <span className="block text-[13px] font-medium">{p.name}</span>
                    <span className="block text-xs text-text-muted">
                      {p.openTasks} open {p.openTasks === 1 ? "task" : "tasks"}
                    </span>
                  </span>
                  <span className="ml-auto text-xs text-text-secondary">
                    {sessions(p.sessionsLeft)} left
                  </span>
                </button>
              ))}
            </div>
            {error && !goalDialog && !projectDialog && (
              <p role="status" className="text-[13px] text-alert">
                {error}
              </p>
            )}
          </section>
        ) : (
          <section className="flex flex-1 flex-col items-center justify-center gap-3 p-10 text-center">
            <GoalRing size={64} stroke={6} percent={0} color="#F2B544" />
            <h2 className="text-lg font-semibold">Set your first goal</h2>
            <p className="max-w-[360px] text-[13px] text-text-secondary">
              A goal is where your sessions add up: what you want, why, and by when. Projects and
              tasks sit under it.
            </p>
            <button
              type="button"
              onClick={() => setGoalDialog("new")}
              className="h-11 rounded-control bg-focus px-5 text-[13px] font-semibold text-focus-ink"
            >
              New goal
            </button>
          </section>
        )}
      </div>

      {goalDialog && (
        <GoalDialog
          goal={goalDialog === "new" ? undefined : goalDialog}
          error={error}
          onClose={closeDialogs}
          onSave={(form) =>
            void run(async () => {
              if (goalDialog === "new") onSelect(await planApi.createGoal(form));
              else await planApi.updateGoal(goalDialog.id, form);
            }).then((ok) => ok && setGoalDialog(null))
          }
          onDelete={
            goalDialog === "new"
              ? undefined
              : () =>
                  void run(() => planApi.deleteGoal(goalDialog.id)).then(
                    (ok) => ok && setGoalDialog(null),
                  )
          }
        />
      )}
      {projectDialog && (
        <ProjectDialog
          project={projectDialog === "new" ? undefined : projectDialog}
          goals={view.goals}
          defaultGoalId={goal?.id ?? null}
          error={error}
          onClose={closeDialogs}
          onSave={(form) =>
            void run(() =>
              projectDialog === "new"
                ? planApi.createProject(form)
                : planApi.updateProject(projectDialog.id, form),
            ).then((ok) => ok && setProjectDialog(null))
          }
          onArchive={
            projectDialog === "new"
              ? undefined
              : () =>
                  void run(() => planApi.archiveProject(projectDialog.id)).then(
                    (ok) => ok && setProjectDialog(null),
                  )
          }
        />
      )}
    </>
  );
}
