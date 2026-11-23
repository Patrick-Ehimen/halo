import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { MainWindow } from "./MainWindow";

const mocks = vi.hoisted(async () => (await import("../../test/timerMocks")).makeMocks());

vi.mock("../../features/app/api", async () => ({
  getAppInfo: vi.fn().mockResolvedValue({ version: "0.1.0", schemaVersion: 1 }),
  appApi: (await mocks).appApi,
}));

vi.mock("../../features/focus/api", async () => {
  const m = await mocks;
  m.timerApi.todayFocus.mockResolvedValue((await import("../../test/timerMocks")).today);
  return { timerApi: m.timerApi, subscribeTimer: m.subscribeTimer };
});

vi.mock("../../features/focus/FocusScreen", () => ({
  FocusScreen: () => <p>focus screen</p>,
}));

vi.mock("../../features/plan/api", () => ({
  planApi: { view: vi.fn().mockResolvedValue(null) },
  onPlanChanged: () => () => {},
}));

describe("MainWindow", () => {
  it("renders the sidebar and the focus screen", async () => {
    render(<MainWindow />);
    expect(screen.getByRole("navigation", { name: "Main" })).toBeInTheDocument();
    expect(screen.getByText("focus screen")).toBeInTheDocument();
    expect(await screen.findByText("v0.1.0")).toBeInTheDocument();
    expect(await screen.findByText("4-day streak")).toBeInTheDocument();
    expect(screen.getByText("Best: 9 days")).toBeInTheDocument();
  });
});
