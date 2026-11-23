import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useFocusStore } from "../../features/focus/store";
import { BreakOverlayWindow } from "./BreakOverlayWindow";

const mocks = vi.hoisted(async () => (await import("../../test/timerMocks")).makeMocks());

vi.mock("../../features/focus/api", async () => {
  const m = await mocks;
  return { timerApi: m.timerApi, subscribeTimer: m.subscribeTimer };
});
vi.mock("../../features/app/api", async () => ({ appApi: (await mocks).appApi }));

describe("BreakOverlayWindow", () => {
  beforeEach(async () => {
    const { running } = await import("../../test/timerMocks");
    useFocusStore.setState({
      snapshot: null,
      today: null,
      settings: null,
      prefs: null,
      error: null,
    });
    (await mocks).timerApi.state.mockResolvedValue({
      ...running,
      phase: "shortBreak",
      plannedMs: 600_000,
      elapsedMs: 178_000,
      remainingMs: 422_000,
      progressPermille: 297,
      intention: "",
    });
  });

  it("counts down the break and offers both ways out", async () => {
    const m = await mocks;
    m.timerApi.end.mockResolvedValue(null);
    render(<BreakOverlayWindow />);

    expect(await screen.findByRole("timer")).toHaveTextContent("07:02");
    expect(screen.getByText("Short break")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "I’m stretching" }));
    expect(m.appApi.closeBreakOverlay).toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Skip break" }));
    expect(m.timerApi.end).toHaveBeenCalled();
  });
});
