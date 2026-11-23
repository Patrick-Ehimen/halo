import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { plan } from "../../test/planFixture";
import { running } from "../../test/timerMocks";
import { useFocusStore } from "../focus/store";
import { PlanScreen } from "./PlanScreen";
import { usePlanStore } from "./store";

const mocks = vi.hoisted(() => ({
  planApi: {
    view: vi.fn(),
    preview: vi.fn(),
    quickAdd: vi.fn(),
    completeTask: vi.fn(),
    reopenTask: vi.fn(),
    createTask: vi.fn(),
    updateTask: vi.fn(),
    deleteTask: vi.fn(),
    createBlock: vi.fn(),
    deleteBlock: vi.fn(),
    startTask: vi.fn(),
  },
  timerApi: {
    state: vi.fn(),
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
  },
}));

vi.mock("./api", () => ({ planApi: mocks.planApi, onPlanChanged: () => () => {} }));
vi.mock("../focus/api", () => ({ timerApi: mocks.timerApi, subscribeTimer: () => () => {} }));
vi.mock("../app/api", () => ({ appApi: { getPrefs: vi.fn().mockResolvedValue(null) } }));

describe("PlanScreen", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    usePlanStore.setState({ view: null, error: null });
    useFocusStore.setState({
      snapshot: null,
      today: null,
      settings: null,
      prefs: null,
      error: null,
    });
    mocks.planApi.view.mockResolvedValue(plan);
    mocks.timerApi.state.mockResolvedValue({ ...running, status: "idle", intention: "" });
  });

  it("groups today's tasks by goal and shows the goal strip", async () => {
    render(<PlanScreen onOpenGoals={() => {}} />);
    expect(
      await screen.findByText("Saturday, 21 November · 3 sessions planned toward 1 goal"),
    ).toBeInTheDocument();

    const strip = screen.getByRole("region", { name: "Today moves your goals" });
    expect(within(strip).getByText("+3 today")).toBeInTheDocument();
    expect(within(strip).getByText("On pace")).toBeInTheDocument();

    const tasks = screen.getByRole("region", { name: "Tasks by goal" });
    expect(within(tasks).getByRole("heading", { name: "Ship meridian-rs v1" })).toBeInTheDocument();
    expect(within(tasks).getByRole("heading", { name: "No goal" })).toBeInTheDocument();
    expect(within(tasks).getByText("Order book benchmarks")).toBeInTheDocument();
    expect(within(tasks).getByText("Errands · Repeats daily")).toBeInTheDocument();

    const timeline = screen.getByRole("region", { name: "Time blocks" });
    expect(within(timeline).getByText("Deep work")).toBeInTheDocument();
  });

  it("completes, starts and quick-adds", async () => {
    mocks.planApi.completeTask.mockResolvedValue(null);
    mocks.planApi.startTask.mockResolvedValue(running);
    mocks.planApi.preview.mockResolvedValue({
      title: "review PR",
      dateLabel: "Tomorrow",
      timeLabel: "10:00",
      estimate: 2,
      projectName: "meridian-rs",
      projectExists: true,
      goalTitle: "Ship meridian-rs v1",
      recurrenceLabel: null,
    });
    mocks.planApi.quickAdd.mockResolvedValue("t3");
    render(<PlanScreen onOpenGoals={() => {}} />);

    fireEvent.click(await screen.findByRole("checkbox", { name: "Complete Buy milk" }));
    await vi.waitFor(() => expect(mocks.planApi.completeTask).toHaveBeenCalledWith("t2"));

    await vi.waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Start Write matching-engine tests" }),
      ).toBeInTheDocument(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Start Write matching-engine tests" }));
    await vi.waitFor(() => expect(mocks.planApi.startTask).toHaveBeenCalledWith("t1", "deep"));

    const input = screen.getByRole("textbox", { name: "Quick add a task" });
    fireEvent.change(input, { target: { value: "review PR tomorrow 10am ~2 #meridian-rs" } });
    expect(await screen.findByText("→ Ship meridian-rs v1")).toBeInTheDocument();
    fireEvent.keyDown(input, { key: "Enter" });
    await vi.waitFor(() =>
      expect(mocks.planApi.quickAdd).toHaveBeenCalledWith(
        "review PR tomorrow 10am ~2 #meridian-rs",
      ),
    );
  });

  it("edits a task in a dialog", async () => {
    mocks.planApi.updateTask.mockResolvedValue(null);
    render(<PlanScreen onOpenGoals={() => {}} />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit Write matching-engine tests" }),
    );
    const dialog = screen.getByRole("dialog", { name: "Edit task" });
    expect(within(dialog).getByLabelText("Time")).toHaveValue("09:30");
    fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
    await vi.waitFor(() =>
      expect(mocks.planApi.updateTask).toHaveBeenCalledWith(
        "t1",
        expect.objectContaining({
          title: "Write matching-engine tests",
          dueTime: "09:30",
          milestoneId: "m2",
        }),
      ),
    );
  });
});
