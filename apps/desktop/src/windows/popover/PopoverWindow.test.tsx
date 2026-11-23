import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { TimerSnapshot } from "../../features/focus/api";
import { useFocusStore } from "../../features/focus/store";
import { PopoverWindow } from "./PopoverWindow";

const idle: TimerSnapshot = {
  phase: "focus",
  status: "idle",
  kind: "deep",
  plannedMs: 50 * 60_000,
  elapsedMs: 0,
  remainingMs: 50 * 60_000,
  overflowing: false,
  earnedBreakMs: 0,
  progressPermille: 0,
  completedInCycle: 1,
  longBreakEvery: 4,
  pausesLeft: 2,
  overflowEnabled: true,
  autoStartBreaks: true,
  intention: "",
  taskId: null,
  note: "",
};

const running: TimerSnapshot = {
  ...idle,
  status: "running",
  elapsedMs: 31 * 60_000 + 36_000,
  remainingMs: 18 * 60_000 + 24_000,
  progressPermille: 632,
  intention: "Write matching-engine tests",
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
    todayFocus: vi.fn().mockResolvedValue({
      focusedSecs: 0,
      sessions: 0,
      focusedLabel: "0m",
      goalLabel: "of your 5h goal",
      goalPermille: 0,
      tree: "seed",
      streak: 0,
      streakLongest: 0,
      streakToday: false,
    }),
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

describe("PopoverWindow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useFocusStore.setState({ snapshot: null, today: null, settings: null, error: null });
    mocks.timerApi.todayFocus.mockResolvedValue({
      focusedSecs: 12_000,
      sessions: 3,
      focusedLabel: "3h 20m",
      goalLabel: "of your 5h goal",
      goalPermille: 666,
      tree: "sapling",
      streak: 4,
      streakLongest: 9,
      streakToday: true,
    });
    mocks.timerApi.settings.mockResolvedValue(null);
    mocks.appApi.miniVisible.mockResolvedValue(false);
    mocks.appApi.setMiniVisible.mockResolvedValue(undefined);
  });

  it("offers quick start when idle", async () => {
    mocks.timerApi.state.mockResolvedValue(idle);
    mocks.timerApi.start.mockResolvedValue(running);
    render(<PopoverWindow />);

    expect(await screen.findByRole("timer")).toHaveTextContent("50:00");
    expect(await screen.findByText("3h 20m · 3 sessions")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Deep 50" }));
    expect(mocks.timerApi.start).toHaveBeenCalledWith("deep", "");
  });

  it("controls a running session", async () => {
    mocks.timerApi.state.mockResolvedValue(running);
    mocks.timerApi.pause.mockResolvedValue({ ...running, status: "paused" });
    render(<PopoverWindow />);

    expect(await screen.findByRole("timer")).toHaveTextContent("18:24");
    expect(screen.getByText("Write matching-engine tests")).toBeInTheDocument();
    expect(screen.getByText("Session 2 of 4")).toBeInTheDocument();
    expect(screen.queryByRole("region", { name: "Quick start" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Pause" }));
    expect(await screen.findByRole("button", { name: "Resume" })).toBeInTheDocument();
  });

  it("opens Halo, quits, and closes on Escape", async () => {
    mocks.timerApi.state.mockResolvedValue(idle);
    render(<PopoverWindow />);
    await screen.findByRole("timer");

    fireEvent.click(screen.getByRole("button", { name: "Open Halo" }));
    fireEvent.click(screen.getByRole("button", { name: "Quit" }));
    fireEvent.keyDown(document, { key: "Escape" });

    expect(mocks.appApi.openMainWindow).toHaveBeenCalled();
    expect(mocks.appApi.quit).toHaveBeenCalled();
    expect(mocks.appApi.hidePopover).toHaveBeenCalled();
  });

  it("shows and hides the mini timer", async () => {
    mocks.timerApi.state.mockResolvedValue(idle);
    render(<PopoverWindow />);

    fireEvent.click(await screen.findByRole("button", { name: "Mini timer" }));
    expect(mocks.appApi.setMiniVisible).toHaveBeenCalledWith(true);
    expect(await screen.findByRole("button", { name: "Hide mini" })).toBeInTheDocument();
  });
});
