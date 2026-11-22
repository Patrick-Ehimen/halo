import { create } from "zustand";
import {
  subscribeTimer,
  timerApi,
  type PresetId,
  type SessionKind,
  type TimerSettings,
  type TimerSnapshot,
  type TodayFocus,
} from "./api";

type FocusState = {
  snapshot: TimerSnapshot | null;
  today: TodayFocus | null;
  settings: TimerSettings | null;
  error: string | null;
  /** Loads the current state and follows timer events. Returns a cleanup. */
  connect: () => () => void;
  start: (preset: PresetId, intention: string) => Promise<void>;
  selectPreset: (preset: PresetId) => Promise<void>;
  startPending: () => Promise<void>;
  pause: () => Promise<void>;
  resume: () => Promise<void>;
  extend: (minutes: number) => Promise<void>;
  end: () => Promise<void>;
  saveSettings: (settings: TimerSettings) => Promise<void>;
  /** Resolves to an error message, or null once saved. */
  logSession: (
    kind: SessionKind,
    started: string,
    minutes: number,
    intention: string,
  ) => Promise<string | null>;
  clearError: () => void;
};

/** UI state for the timer. Rust owns the time; this only mirrors its events. */
export const useFocusStore = create<FocusState>((set, get) => {
  const run = async (action: () => Promise<TimerSnapshot>) => {
    try {
      set({ snapshot: await action(), error: null });
    } catch (e) {
      set({ error: e instanceof Error ? e.message : String(e) });
    }
  };
  const refreshToday = async () => {
    try {
      set({ today: await timerApi.todayFocus() });
    } catch {
      // The today card is optional; leave the last value.
    }
  };

  return {
    snapshot: null,
    today: null,
    settings: null,
    error: null,
    connect: () => {
      void run(timerApi.state);
      void refreshToday();
      void timerApi
        .settings()
        .then((settings) => set({ settings }))
        .catch(() => {});
      return subscribeTimer({
        onTick: (snapshot) => set({ snapshot }),
        onPhase: (event) => {
          if (event.type === "focusEnded") void refreshToday();
        },
      });
    },
    start: (preset, intention) => run(() => timerApi.start(preset, intention)),
    selectPreset: (preset) => run(() => timerApi.selectPreset(preset)),
    startPending: () => run(timerApi.startPending),
    pause: () => run(timerApi.pause),
    resume: () => run(timerApi.resume),
    extend: (minutes) => run(() => timerApi.extend(minutes)),
    end: () => run(timerApi.end),
    saveSettings: async (settings) => {
      const previous = get().settings;
      set({ settings });
      try {
        set({ snapshot: await timerApi.setSettings(settings), error: null });
      } catch (e) {
        set({ settings: previous, error: e instanceof Error ? e.message : String(e) });
      }
    },
    logSession: async (kind, started, minutes, intention) => {
      try {
        await timerApi.logSession(kind, started, minutes, intention);
        await refreshToday();
        return null;
      } catch (e) {
        return e instanceof Error ? e.message : String(e);
      }
    },
    clearError: () => set({ error: null }),
  };
});
