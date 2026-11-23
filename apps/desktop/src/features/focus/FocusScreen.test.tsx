import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { TimerSnapshot } from "./api";
import { FocusScreen } from "./FocusScreen";
import { useFocusStore } from "./store";

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

const settings = {
  overflow: true,
  autoStartBreaks: true,
  autoStartFocus: false,
  pauseLimit: 2,
  longBreakEvery: 4,
};

const api = vi.hoisted(() => ({
  timerApi: {
    state: vi.fn(),
    start: vi.fn(),
    selectPreset: vi.fn(),
    startPending: vi.fn(),
    pause: vi.fn(),
    resume: vi.fn(),
    extend: vi.fn(),
    end: vi.fn(),
    todayFocus: vi.fn(),
    settings: vi.fn(),
    setSettings: vi.fn(),
    logSession: vi.fn(),
  },
  subscribeTimer: vi.fn(() => () => {}),
}));

vi.mock("./api", () => api);
vi.mock("../plan/api", () => ({
  planApi: { view: vi.fn().mockResolvedValue(null), startTask: vi.fn() },
  onPlanChanged: () => () => {},
}));
vi.mock("../app/api", () => ({
  appApi: {
    getPrefs: vi.fn().mockResolvedValue(null),
    setPrefs: vi.fn(),
    togglePalette: vi.fn(),
    onUiRequest: vi.fn(() => () => {}),
  },
}));

describe("FocusScreen", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useFocusStore.setState({ snapshot: null, today: null, settings: null, error: null });
    api.timerApi.todayFocus.mockResolvedValue({ focusedSecs: 12_000, sessions: 3 });
    api.timerApi.settings.mockResolvedValue(settings);
  });

  it("shows the idle timer and starts a session with the intention", async () => {
    api.timerApi.state.mockResolvedValue(idle);
    api.timerApi.start.mockResolvedValue(running);
    render(<FocusScreen />);

    expect(await screen.findByRole("timer")).toHaveTextContent("50:00");
    expect(screen.getByText("Session 2 of 4 · long break after 4")).toBeInTheDocument();
    expect(await screen.findByText("3h 20m")).toBeInTheDocument();

    fireEvent.change(screen.getByPlaceholderText("What will you focus on?"), {
      target: { value: "Write matching-engine tests" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Start focus" }));

    expect(api.timerApi.start).toHaveBeenCalledWith("deep", "Write matching-engine tests");
    expect(await screen.findByRole("button", { name: "Pause" })).toBeInTheDocument();
    expect(screen.getByRole("timer")).toHaveTextContent("18:24");
    expect(screen.getByText("Write matching-engine tests")).toBeInTheDocument();
  });

  it("shows overflow past the goal", async () => {
    api.timerApi.state.mockResolvedValue({
      ...running,
      elapsedMs: 53 * 60_000,
      remainingMs: -3 * 60_000,
      overflowing: true,
      progressPermille: 1000,
    });
    render(<FocusScreen />);
    expect(await screen.findByRole("timer")).toHaveTextContent("+03:00");
    expect(screen.getByText("Past your 50 min goal")).toBeInTheDocument();
  });

  it("disables pause when none are left", async () => {
    api.timerApi.state.mockResolvedValue({ ...running, pausesLeft: 0 });
    render(<FocusScreen />);
    expect(await screen.findByRole("button", { name: "Pause" })).toBeDisabled();
    expect(screen.getByText("0 pauses left")).toBeInTheDocument();
  });

  it("shows a refused action", async () => {
    api.timerApi.state.mockResolvedValue(running);
    api.timerApi.end.mockRejectedValue(new Error("no session is in progress"));
    render(<FocusScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "End" }));
    expect(await screen.findByText("no session is in progress")).toBeInTheDocument();
  });

  it("saves a changed setting", async () => {
    api.timerApi.state.mockResolvedValue(idle);
    api.timerApi.setSettings.mockResolvedValue({ ...idle, overflowEnabled: false });
    render(<FocusScreen />);

    const button = await screen.findByRole("button", { name: "Timer settings" });
    await waitFor(() => expect(button).toBeEnabled());
    fireEvent.click(button);
    fireEvent.click(screen.getByRole("switch", { name: /Overflow/ }));

    expect(api.timerApi.setSettings).toHaveBeenCalledWith({ ...settings, overflow: false });
    expect(await screen.findByText("Overflow off — ends at the goal")).toBeInTheDocument();
  });

  it("logs a missed session and shows a refusal", async () => {
    api.timerApi.state.mockResolvedValue(idle);
    api.timerApi.logSession
      .mockRejectedValueOnce(new Error("That session hasn't ended yet"))
      .mockResolvedValueOnce(null);
    render(<FocusScreen />);

    fireEvent.click(await screen.findByRole("button", { name: "Log a missed session" }));
    const dialog = screen.getByRole("dialog", { name: "Log a missed session" });
    fireEvent.change(within(dialog).getByLabelText("Started"), {
      target: { value: "2026-11-22T09:30" },
    });
    fireEvent.change(within(dialog).getByLabelText("Intention"), {
      target: { value: "Review PR" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Log session" }));

    expect(await screen.findByText("That session hasn't ended yet")).toBeInTheDocument();
    expect(api.timerApi.logSession).toHaveBeenCalledWith(
      "deep",
      "2026-11-22T09:30",
      50,
      "Review PR",
    );

    fireEvent.click(screen.getByRole("button", { name: "Log session" }));
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
  });

  it("picks a pause limit from the custom dropdown", async () => {
    api.timerApi.state.mockResolvedValue(idle);
    api.timerApi.setSettings.mockResolvedValue({ ...idle, pausesLeft: null });
    render(<FocusScreen />);

    const button = await screen.findByRole("button", { name: "Timer settings" });
    await waitFor(() => expect(button).toBeEnabled());
    fireEvent.click(button);
    fireEvent.click(screen.getByRole("button", { name: /Pauses per session/ }));
    fireEvent.click(screen.getByRole("option", { name: "Unlimited" }));

    expect(api.timerApi.setSettings).toHaveBeenCalledWith({ ...settings, pauseLimit: null });
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
    expect(screen.getByRole("dialog", { name: "Timer settings" })).toBeInTheDocument();
  });

  it("closes settings on a click outside, but not inside", async () => {
    api.timerApi.state.mockResolvedValue(idle);
    api.timerApi.setSettings.mockResolvedValue(idle);
    render(<FocusScreen />);

    const button = await screen.findByRole("button", { name: "Timer settings" });
    await waitFor(() => expect(button).toBeEnabled());
    fireEvent.click(button);
    const panel = screen.getByRole("dialog", { name: "Timer settings" });

    fireEvent.pointerDown(screen.getByRole("switch", { name: /Overflow/ }));
    expect(panel).toBeInTheDocument();

    fireEvent.pointerDown(screen.getByRole("timer"));
    expect(screen.queryByRole("dialog", { name: "Timer settings" })).not.toBeInTheDocument();
  });

  it("closes only the dropdown on Escape, then the panel", async () => {
    api.timerApi.state.mockResolvedValue(idle);
    render(<FocusScreen />);

    const button = await screen.findByRole("button", { name: "Timer settings" });
    await waitFor(() => expect(button).toBeEnabled());
    fireEvent.click(button);
    fireEvent.click(screen.getByRole("button", { name: /Long break every/ }));
    fireEvent.keyDown(screen.getByRole("listbox"), { key: "Escape" });

    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
    expect(screen.getByRole("dialog", { name: "Timer settings" })).toBeInTheDocument();

    fireEvent.keyDown(document.body, { key: "Escape" });
    expect(screen.queryByRole("dialog", { name: "Timer settings" })).not.toBeInTheDocument();
  });
});
