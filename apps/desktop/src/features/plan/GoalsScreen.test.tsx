import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { plan } from "../../test/planFixture";
import { running } from "../../test/timerMocks";
import { useFocusStore } from "../focus/store";
import { GoalsScreen } from "./GoalsScreen";
import { usePlanStore } from "./store";

const mocks = vi.hoisted(() => ({
  planApi: {
    view: vi.fn(),
    createGoal: vi.fn(),
    updateGoal: vi.fn(),
    setGoalDone: vi.fn(),
    deleteGoal: vi.fn(),
    createMilestone: vi.fn(),
    setMilestoneDone: vi.fn(),
    deleteMilestone: vi.fn(),
    createProject: vi.fn(),
    updateProject: vi.fn(),
    archiveProject: vi.fn(),
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

describe("GoalsScreen", () => {
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

  it("shows a goal's progress, pace, milestones and today", async () => {
    render(<GoalsScreen selectedId="g1" onSelect={() => {}} onOpenPlan={() => {}} />);
    const detail = await screen.findByRole("region", { name: "Ship meridian-rs v1" });
    expect(within(detail).getByText("46 of 120")).toBeInTheDocument();
    expect(within(detail).getByText("7 a week")).toBeInTheDocument();
    expect(within(detail).getByText("4.5 needed to finish on time")).toBeInTheDocument();
    expect(within(detail).getByText("16 weeks left")).toBeInTheDocument();
    expect(within(detail).getByText("1 of 2 done")).toBeInTheDocument();
    expect(within(detail).getByText("+3 sessions")).toBeInTheDocument();
    expect(within(detail).getByText("Write matching-engine tests")).toBeInTheDocument();
  });

  it("ticks a milestone and adds one", async () => {
    mocks.planApi.setMilestoneDone.mockResolvedValue(null);
    mocks.planApi.createMilestone.mockResolvedValue("m3");
    render(<GoalsScreen selectedId="g1" onSelect={() => {}} onOpenPlan={() => {}} />);

    fireEvent.click(
      await screen.findByRole("checkbox", { name: "Mark Order book benchmarks done" }),
    );
    await vi.waitFor(() => expect(mocks.planApi.setMilestoneDone).toHaveBeenCalledWith("m2", true));

    const input = screen.getByRole("textbox", { name: "New milestone" });
    fireEvent.change(input, { target: { value: "Public beta" } });
    fireEvent.submit(input.closest("form") as HTMLFormElement);
    await vi.waitFor(() =>
      expect(mocks.planApi.createMilestone).toHaveBeenCalledWith("g1", "Public beta", null),
    );
  });

  it("creates a goal", async () => {
    mocks.planApi.createGoal.mockResolvedValue("g2");
    const onSelect = vi.fn();
    render(<GoalsScreen selectedId="g1" onSelect={onSelect} onOpenPlan={() => {}} />);
    fireEvent.click((await screen.findAllByRole("button", { name: "New goal" }))[0] as HTMLElement);
    const dialog = screen.getByRole("dialog", { name: "New goal" });
    fireEvent.change(within(dialog).getByLabelText("What do you want to achieve?"), {
      target: { value: "Land a Rust role" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Add goal" }));
    await vi.waitFor(() =>
      expect(mocks.planApi.createGoal).toHaveBeenCalledWith(
        expect.objectContaining({ title: "Land a Rust role", targetSessions: 60 }),
      ),
    );
    expect(onSelect).toHaveBeenCalledWith("g2");
  });

  it("invites a first goal when there are none", async () => {
    mocks.planApi.view.mockResolvedValue({ ...plan, goals: [], projects: [] });
    render(<GoalsScreen selectedId={null} onSelect={() => {}} onOpenPlan={() => {}} />);
    expect(await screen.findByRole("heading", { name: "Set your first goal" })).toBeInTheDocument();
  });
});
