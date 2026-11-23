import { vi } from "vitest";
import type { TimerSnapshot, TodayFocus } from "../features/focus/api";

/** A running Deep 50 session, 18:24 left. */
export const running: TimerSnapshot = {
  phase: "focus",
  status: "running",
  kind: "deep",
  plannedMs: 50 * 60_000,
  elapsedMs: 31 * 60_000 + 36_000,
  remainingMs: 18 * 60_000 + 24_000,
  overflowing: false,
  earnedBreakMs: 0,
  progressPermille: 632,
  completedInCycle: 1,
  longBreakEvery: 4,
  pausesLeft: 2,
  overflowEnabled: true,
  autoStartBreaks: true,
  intention: "Write matching-engine tests",
  taskId: null,
  note: "",
};

/** 3h 20m of a 5h goal, a 4-day streak. */
export const today: TodayFocus = {
  focusedSecs: 12_000,
  sessions: 3,
  focusedLabel: "3h 20m",
  goalLabel: "of your 5h goal",
  goalPermille: 666,
  tree: "sapling",
  streak: 4,
  streakLongest: 9,
  streakToday: true,
};

/** Nothing yet today. */
export const emptyToday: TodayFocus = {
  ...today,
  focusedSecs: 0,
  sessions: 0,
  focusedLabel: "0m",
  goalPermille: 0,
  tree: "seed",
  streak: 0,
  streakLongest: 0,
  streakToday: false,
};

/** Fresh mocks for the focus and app APIs. Use inside `vi.hoisted`. */
export function makeMocks() {
  return {
    timerApi: {
      state: vi.fn(),
      start: vi.fn(),
      selectPreset: vi.fn(),
      startPending: vi.fn(),
      pause: vi.fn(),
      resume: vi.fn(),
      extend: vi.fn(),
      end: vi.fn(),
      todayFocus: vi.fn().mockResolvedValue(emptyToday),
      settings: vi.fn().mockResolvedValue(null),
      setSettings: vi.fn(),
      logSession: vi.fn(),
      checkIn: vi.fn(),
    },
    subscribeTimer: vi.fn(() => () => {}),
    appApi: {
      getPrefs: vi.fn().mockResolvedValue(null),
      setPrefs: vi.fn(),
      openMainWindow: vi.fn(),
      quit: vi.fn(),
      hidePopover: vi.fn(),
      fitWindow: vi.fn(),
      miniVisible: vi.fn().mockResolvedValue(false),
      setMiniVisible: vi.fn(),
      togglePalette: vi.fn(),
      hidePalette: vi.fn().mockResolvedValue(null),
      requestUi: vi.fn(),
      dismissCheckIn: vi.fn(),
      closeBreakOverlay: vi.fn(),
      onUiRequest: vi.fn(() => () => {}),
    },
  };
}
