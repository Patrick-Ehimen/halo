import {
  commands,
  type ExportFormat,
  type IpcError,
  type ReflectionInput_Deserialize,
  type ReflectionView_Serialize,
  type StatsRange,
  type StatsView_Serialize,
  type TreeStageView,
} from "@halo/core";

export type StatsView = StatsView_Serialize;
export type ReflectionView = ReflectionView_Serialize;
export type ReflectionInput = Required<ReflectionInput_Deserialize>;
export type { ExportFormat, StatsRange, TreeStageView };

type CommandResult<T> = { status: "ok"; data: T } | { status: "error"; error: IpcError };

async function unwrap<T>(call: Promise<CommandResult<T>>): Promise<T> {
  const result = await call;
  if (result.status === "error") throw new Error(result.error.message);
  return result.data;
}

export const statsApi = {
  view: (range: StatsRange) => unwrap(commands.statsView(range)),
  /** Writes every session to Downloads; resolves to the file's path. */
  exportSessions: (format: ExportFormat) => unwrap(commands.exportSessions(format)),
  reflection: () => unwrap(commands.reflectionState()),
  reflect: (input: ReflectionInput) => unwrap(commands.reflect(input)),
};
