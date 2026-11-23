import {
  commands,
  events,
  type AppInfo,
  type IpcError,
  type Prefs_Serialize,
  type UiAction,
} from "@halo/core";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type { AppInfo, UiAction };
export type Prefs = Prefs_Serialize;

type CommandResult<T> = { status: "ok"; data: T } | { status: "error"; error: IpcError };

async function unwrap<T>(call: Promise<CommandResult<T>>): Promise<T> {
  const result = await call;
  if (result.status === "error") throw new Error(result.error.message);
  return result.data;
}

export async function getAppInfo(): Promise<AppInfo> {
  const result = await commands.appInfo();
  if (result.status === "error") {
    throw new Error(result.error.message);
  }
  return result.data;
}

export const appApi = {
  getPrefs: () => commands.getPrefs(),
  setPrefs: (prefs: Prefs) => unwrap(commands.setPrefs(prefs)),
  togglePalette: () => unwrap(commands.togglePalette()),
  hidePalette: () => unwrap(commands.hidePalette()),
  /** Opens a panel in the main window (and shows it). */
  requestUi: (action: UiAction) => unwrap(commands.requestUi(action)),
  dismissCheckIn: () => unwrap(commands.dismissCheckIn()),
  closeBreakOverlay: () => unwrap(commands.closeBreakOverlay()),
  /** Follows requests from the menu bar or palette to open a panel here. */
  onUiRequest: (handler: (action: UiAction) => void): (() => void) => {
    const pending = events.uiRequest.listen((e) => handler(e.payload.action));
    return () => void pending.then((unlisten) => unlisten());
  },
  openMainWindow: () => commands.openMainWindow(),
  quit: () => commands.quitApp(),
  /** Resizes the current window to fit its content (popover, mini timer). */
  fitWindow: (width: number, height: number) =>
    getCurrentWindow().setSize(new LogicalSize(width, Math.ceil(height))),
  miniVisible: () => commands.miniVisible(),
  setMiniVisible: async (visible: boolean) => {
    const result = await commands.setMiniVisible(visible);
    if (result.status === "error") throw new Error(result.error.message);
  },
  hidePopover: async () => {
    const result = await commands.hidePopover();
    if (result.status === "error") throw new Error(result.error.message);
  },
};
