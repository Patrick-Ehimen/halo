import { vi } from "vitest";
import type { TimerSnapshot } from "../features/focus/api";

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
      todayFocus: vi.fn().mockResolvedValue({ focusedSecs: 0, sessions: 0 }),
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
