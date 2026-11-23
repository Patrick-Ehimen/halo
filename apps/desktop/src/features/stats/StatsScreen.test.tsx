import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { statsWeek } from "../../test/statsFixture";
import { StatsScreen } from "./StatsScreen";
import { useStatsStore } from "./store";

const api = vi.hoisted(() => ({
  statsApi: {
    view: vi.fn(),
    exportSessions: vi.fn(),
    reflection: vi.fn(),
    reflect: vi.fn(),
  },
}));
vi.mock("./api", () => api);
vi.mock("../plan/api", () => ({ onPlanChanged: () => () => {} }));
vi.mock("../focus/api", () => ({ subscribeTimer: () => () => {} }));

describe("StatsScreen", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useStatsStore.setState({ range: "week", view: null, error: null, exported: null });
    api.statsApi.view.mockResolvedValue(statsWeek);
  });

  it("shows the week: KPIs, heatmap, peak hours, projects, achievements and the grove", async () => {
    render(<StatsScreen />);
    expect(await screen.findByText("21h 40m")).toBeInTheDocument();
    expect(api.statsApi.view).toHaveBeenCalledWith("week");
    expect(screen.getByText("23–29 Nov")).toBeInTheDocument();
    expect(screen.getByText("+6 vs last week")).toHaveClass("text-shield");
    expect(screen.getByText("Best: 09:00–11:00")).toBeInTheDocument();
    expect(screen.getAllByRole("listitem", { name: /^w\d+d\d+$/ })).toHaveLength(24 * 7);
    expect(screen.getByText("meridian-rs")).toBeInTheDocument();
    expect(screen.getByText("Earned 14 Nov 2026")).toBeInTheDocument();
    expect(screen.getByText("6 of 10")).toBeInTheDocument();
    expect(screen.getByText("Today")).toBeInTheDocument();
    expect(screen.getAllByRole("img", { name: "A tree in bloom" }).length).toBeGreaterThan(0);
    expect(screen.queryByText("Weekly report")).not.toBeInTheDocument();
  });

  it("switches range and exports", async () => {
    api.statsApi.exportSessions.mockResolvedValue(
      "/Users/me/Downloads/halo-sessions-2026-11-23.csv",
    );
    render(<StatsScreen />);
    await screen.findByText("21h 40m");

    fireEvent.click(screen.getByRole("button", { name: "Month" }));
    expect(api.statsApi.view).toHaveBeenLastCalledWith("month");
    expect(screen.getByRole("button", { name: "Month" })).toHaveAttribute("aria-pressed", "true");

    fireEvent.click(screen.getByRole("button", { name: "Export CSV" }));
    expect(api.statsApi.exportSessions).toHaveBeenCalledWith("csv");
    expect(
      await screen.findByText("Saved to /Users/me/Downloads/halo-sessions-2026-11-23.csv"),
    ).toBeInTheDocument();
  });

  it("shows an error when the view can't load", async () => {
    api.statsApi.view.mockRejectedValue(new Error("database: locked"));
    render(<StatsScreen />);
    expect(await screen.findByText("database: locked")).toBeInTheDocument();
  });
});
