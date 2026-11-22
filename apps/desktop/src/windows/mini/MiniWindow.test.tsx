import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { TimerSnapshot } from "../../features/focus/api";
import { useFocusStore } from "../../features/focus/store";
import { MiniWindow } from "./MiniWindow";

const running: TimerSnapshot = {
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
  intention: "Matching-engine tests",
};

const mocks = vi.hoisted(() => ({
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
}));

vi.mock("../../features/focus/api", () => ({
  timerApi: mocks.timerApi,
  subscribeTimer: mocks.subscribeTimer,
}));
vi.mock("../../features/app/api", () => ({ appApi: mocks.appApi }));

describe("MiniWindow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useFocusStore.setState({ snapshot: null, today: null, settings: null, error: null });
    mocks.timerApi.todayFocus.mockResolvedValue({ focusedSecs: 0, sessions: 0 });
    mocks.timerApi.settings.mockResolvedValue(null);
  });

  it("shows the session and pauses it", async () => {
    mocks.timerApi.state.mockResolvedValue(running);
    mocks.timerApi.pause.mockResolvedValue({ ...running, status: "paused" });
    render(<MiniWindow />);

    expect(await screen.findByRole("timer")).toHaveTextContent("18:24");
    expect(screen.getByText("Matching-engine tests")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Pause" }));
    expect(await screen.findByRole("button", { name: "Resume" })).toBeInTheDocument();
  });

  it("starts the current preset when idle", async () => {
    mocks.timerApi.state.mockResolvedValue({ ...running, status: "idle", intention: "" });
    mocks.timerApi.start.mockResolvedValue(running);
    render(<MiniWindow />);

    fireEvent.click(await screen.findByRole("button", { name: "Start focus" }));
    expect(mocks.timerApi.start).toHaveBeenCalledWith("deep", "");
  });

  it("shows a break in blue with its own label", async () => {
    mocks.timerApi.state.mockResolvedValue({
      ...running,
      phase: "shortBreak",
      plannedMs: 600_000,
      elapsedMs: 178_000,
      remainingMs: 422_000,
      intention: "",
    });
    render(<MiniWindow />);
    expect(await screen.findByRole("timer")).toHaveTextContent("07:02");
    expect(screen.getByText("Short break")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Hide mini timer" })).not.toBeInTheDocument();
  });
});
