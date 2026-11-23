import {
  commands,
  events,
  type IpcError,
  type CheckInResponse,
  type PresetId,
  type SessionKind,
  type TimerPhaseEvent,
  type TimerSettings_Serialize,
  type TimerSnapshot_Serialize,
  type TodayFocus,
} from "@halo/core";

export type TimerSnapshot = TimerSnapshot_Serialize;
export type TimerSettings = TimerSettings_Serialize;
export type { CheckInResponse, PresetId, SessionKind, TimerPhaseEvent, TodayFocus };

type CommandResult<T> = { status: "ok"; data: T } | { status: "error"; error: IpcError };

async function unwrap<T>(call: Promise<CommandResult<T>>): Promise<T> {
  const result = await call;
  if (result.status === "error") throw new Error(result.error.message);
  return result.data;
}

export const timerApi = {
  state: () => unwrap(commands.timerState()),
  start: (preset: PresetId, intention: string) => unwrap(commands.timerStart(preset, intention)),
  selectPreset: (preset: PresetId) => unwrap(commands.timerSelectPreset(preset)),
  startPending: () => unwrap(commands.timerStartPending()),
  pause: () => unwrap(commands.timerPause()),
  resume: () => unwrap(commands.timerResume()),
  extend: (minutes: number) => unwrap(commands.timerExtend(minutes)),
  end: () => unwrap(commands.timerEnd()),
  todayFocus: () => unwrap(commands.todayFocus()),
  checkIn: (response: CheckInResponse) => unwrap(commands.checkIn(response)),
  settings: () => unwrap(commands.timerSettings()),
  setSettings: (settings: TimerSettings) => unwrap(commands.timerSetSettings(settings)),
  /** `started` is a `datetime-local` value; Rust turns it into a timestamp. */
  logSession: (kind: SessionKind, started: string, minutes: number, intention: string) =>
    unwrap(commands.logSession(kind, started, minutes, intention)),
};

/** Subscribes to timer events. Returns a function that unsubscribes. */
export function subscribeTimer(handlers: {
  onTick: (snapshot: TimerSnapshot) => void;
  onPhase: (event: TimerPhaseEvent) => void;
}): () => void {
  const pending = [
    events.timerTick.listen((e) => handlers.onTick(e.payload)),
    events.timerPhase.listen((e) => handlers.onPhase(e.payload)),
  ];
  return () => {
    for (const p of pending) void p.then((unlisten) => unlisten());
  };
}
