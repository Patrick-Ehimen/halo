import { create } from "zustand";
import { appApi, type Prefs } from "../app/api";
import { onPlanChanged } from "../plan/api";
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
  prefs: Prefs | null;
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
  savePrefs: (prefs: Prefs) => Promise<void>;
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
    prefs: null,
    error: null,
    connect: () => {
      void appApi
        .getPrefs()
        .then((prefs) => set({ prefs }))
        .catch(() => {});
      void run(timerApi.state);
      void refreshToday();
      void timerApi
        .settings()
        .then((settings) => set({ settings }))
        .catch(() => {});
      const offTimer = subscribeTimer({
        onTick: (snapshot) => set({ snapshot }),
        onPhase: (event) => {
          if (event.type === "focusEnded") void refreshToday();
        },
      });
      // Reflections and plan changes can move today's numbers too.
      const offPlan = onPlanChanged(() => void refreshToday());
      return () => {
        offTimer();
        offPlan();
      };
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
    savePrefs: async (prefs) => {
      const previous = get().prefs;
      set({ prefs });
      try {
        set({ prefs: await appApi.setPrefs(prefs), error: null });
        // The goal ring and tree follow the daily goal.
        if (prefs.dailyGoalMinutes !== previous?.dailyGoalMinutes) void refreshToday();
      } catch (e) {
        set({ prefs: previous, error: e instanceof Error ? e.message : String(e) });
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
