import {
  commands,
  events,
  type BlockView_Serialize,
  type GoalForm_Deserialize,
  type GoalView_Serialize,
  type IpcError,
  type MilestoneView_Serialize,
  type PaceView,
  type PlanView_Serialize,
  type PresetId,
  type ProjectForm_Deserialize,
  type ProjectView_Serialize,
  type QuickAddPreview_Serialize,
  type TaskForm_Deserialize,
  type TaskView_Serialize,
  type TimerSnapshot_Serialize,
} from "@halo/core";

export type PlanView = PlanView_Serialize;
export type GoalView = GoalView_Serialize;
export type MilestoneView = MilestoneView_Serialize;
export type ProjectView = ProjectView_Serialize;
export type TaskView = TaskView_Serialize;
export type BlockView = BlockView_Serialize;
export type QuickAddPreview = QuickAddPreview_Serialize;
export type GoalForm = GoalForm_Deserialize;
export type ProjectForm = ProjectForm_Deserialize;
export type TaskForm = Required<TaskForm_Deserialize>;
export type { PaceView };

type CommandResult<T> = { status: "ok"; data: T } | { status: "error"; error: IpcError };

async function unwrap<T>(call: Promise<CommandResult<T>>): Promise<T> {
  const result = await call;
  if (result.status === "error") throw new Error(result.error.message);
  return result.data;
}

export const planApi = {
  view: () => unwrap(commands.planView()),
  preview: (text: string) => unwrap(commands.quickAddPreview(text)),
  quickAdd: (text: string) => unwrap(commands.quickAdd(text)),

  createGoal: (form: GoalForm) => unwrap(commands.goalCreate(form)),
  updateGoal: (id: string, form: GoalForm) => unwrap(commands.goalUpdate(id, form)),
  setGoalDone: (id: string, done: boolean) => unwrap(commands.goalSetDone(id, done)),
  deleteGoal: (id: string) => unwrap(commands.goalDelete(id)),

  createMilestone: (goalId: string, title: string, dueOn: string | null) =>
    unwrap(commands.milestoneCreate(goalId, title, dueOn)),
  setMilestoneDone: (id: string, done: boolean) => unwrap(commands.milestoneSetDone(id, done)),
  deleteMilestone: (id: string) => unwrap(commands.milestoneDelete(id)),

  createProject: (form: ProjectForm) => unwrap(commands.projectCreate(form)),
  updateProject: (id: string, form: ProjectForm) => unwrap(commands.projectUpdate(id, form)),
  archiveProject: (id: string) => unwrap(commands.projectArchive(id)),

  createTask: (form: TaskForm) => unwrap(commands.taskCreate(form)),
  updateTask: (id: string, form: TaskForm) => unwrap(commands.taskUpdate(id, form)),
  completeTask: (id: string) => unwrap(commands.taskComplete(id)),
  reopenTask: (id: string) => unwrap(commands.taskReopen(id)),
  deleteTask: (id: string) => unwrap(commands.taskDelete(id)),

  /** `start` is local "HH:MM"; Rust places it on today. */
  createBlock: (taskId: string | null, title: string, start: string, minutes: number) =>
    unwrap(commands.blockCreate(taskId, title, start, minutes)),
  deleteBlock: (id: string) => unwrap(commands.blockDelete(id)),

  /** Starts a focus session on a task. */
  startTask: (taskId: string, preset: PresetId): Promise<TimerSnapshot_Serialize> =>
    unwrap(commands.timerStartTask(taskId, preset)),
};

/** Calls `handler` whenever the plan changes in any window. Returns a cleanup. */
export function onPlanChanged(handler: () => void): () => void {
  const pending = events.planChanged.listen(() => handler());
  return () => void pending.then((unlisten) => unlisten());
}
