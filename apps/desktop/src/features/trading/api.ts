import {
  commands,
  events,
  type IpcError,
  type MarketId,
  type TradingSettings_Serialize,
  type TradingView_Serialize,
} from "@halo/core";

export type TradingView = TradingView_Serialize;
export type TradingSettings = TradingSettings_Serialize;
export type { MarketId };

type CommandResult<T> = { status: "ok"; data: T } | { status: "error"; error: IpcError };

async function unwrap<T>(call: Promise<CommandResult<T>>): Promise<T> {
  const result = await call;
  if (result.status === "error") throw new Error(result.error.message);
  return result.data;
}

export const tradingApi = {
  view: () => unwrap(commands.tradingView()),
  settings: () => unwrap(commands.tradingSettings()),
  setSettings: (s: TradingSettings) => unwrap(commands.tradingSetSettings(s)),
  setChecked: (id: string, checked: boolean) => unwrap(commands.checklistSet(id, checked)),
  addItem: (label: string, hint: string) => unwrap(commands.checklistAdd(label, hint)),
  removeItem: (id: string) => unwrap(commands.checklistRemove(id)),
  resetChecklist: () => unwrap(commands.checklistReset()),
  restoreSuggested: () => unwrap(commands.checklistRestoreSuggested()),
  headCheck: (score: number, factors: string[]) => unwrap(commands.headCheck(score, factors)),
  clearHeadCheck: () => unwrap(commands.headCheckClear()),
  openAutomationSettings: () => unwrap(commands.openAutomationSettings()),
};

/** Follows checklist, head check and settings changes from any window. */
export function onTradingChanged(handler: () => void): () => void {
  const pending = events.tradingChanged.listen(() => handler());
  return () => void pending.then((unlisten) => unlisten());
}
