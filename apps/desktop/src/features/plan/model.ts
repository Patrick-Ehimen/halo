import type { GoalView, PlanView, ProjectView, TaskForm, TaskView } from "./api";

const MUTED = "#8A8E9E";

/** Lookups over one plan view, for drawing. No time logic lives here. */
export function model(view: PlanView) {
  const projects = new Map(view.projects.map((p) => [p.id, p]));
  const goals = new Map(view.goals.map((g) => [g.id, g]));
  const milestones = new Map(view.goals.flatMap((g) => g.milestones).map((m) => [m.id, m]));
  const allTasks = new Map([...view.open, ...view.today].map((t) => [t.id, t]));

  const projectOf = (t: TaskView): ProjectView | undefined =>
    t.projectId ? projects.get(t.projectId) : undefined;
  const goalOf = (t: TaskView): GoalView | undefined => {
    const goalId = projectOf(t)?.goalId;
    return goalId ? goals.get(goalId) : undefined;
  };
  const colorOf = (t: TaskView): string => projectOf(t)?.color ?? goalOf(t)?.color ?? MUTED;

  /** Today's top-level tasks grouped by goal (goal order), then "No goal". */
  const groups = () => {
    const top = view.today.filter((t) => !t.parentId);
    const byGoal = view.goals
      .map((g) => ({
        goal: g as GoalView | null,
        tasks: top.filter((t) => goalOf(t)?.id === g.id),
      }))
      .filter((grp) => grp.tasks.length > 0);
    const loose = top.filter((t) => !goalOf(t));
    return loose.length ? [...byGoal, { goal: null, tasks: loose }] : byGoal;
  };

  const subtasksOf = (t: TaskView) => view.today.filter((s) => s.parentId === t.id);

  return { projects, goals, milestones, allTasks, projectOf, goalOf, colorOf, groups, subtasksOf };
}

/** A task as its edit form. */
export function formOf(t: TaskView): TaskForm & { id: string } {
  return {
    id: t.id,
    title: t.title,
    projectId: t.projectId,
    milestoneId: t.milestoneId,
    parentId: t.parentId,
    estimateSessions: t.estimateSessions,
    plannedOn: t.plannedOn,
    dueTime: t.dueTime,
    recurrence: t.recurrence,
  };
}

/** "3 sessions", "1 session". */
export function sessions(n: number): string {
  return `${n} ${n === 1 ? "session" : "sessions"}`;
}
