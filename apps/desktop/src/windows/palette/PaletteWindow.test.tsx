import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useFocusStore } from "../../features/focus/store";
import { PaletteWindow } from "./PaletteWindow";

const mocks = vi.hoisted(async () => (await import("../../test/timerMocks")).makeMocks());

vi.mock("../../features/focus/api", async () => {
  const m = await mocks;
  return { timerApi: m.timerApi, subscribeTimer: m.subscribeTimer };
});
vi.mock("../../features/app/api", async () => ({ appApi: (await mocks).appApi }));
vi.mock("../../features/plan/api", () => ({
  planApi: { view: vi.fn().mockResolvedValue(null), startTask: vi.fn(), quickAdd: vi.fn() },
  onPlanChanged: () => () => {},
}));

describe("PaletteWindow", () => {
  beforeEach(async () => {
    const { running } = await import("../../test/timerMocks");
    useFocusStore.setState({
      snapshot: null,
      today: null,
      settings: null,
      prefs: null,
      error: null,
    });
    (await mocks).timerApi.state.mockResolvedValue({ ...running, status: "idle", intention: "" });
  });

  it("starts a quick start typed as text", async () => {
    const m = await mocks;
    m.timerApi.start.mockResolvedValue(null);
    render(<PaletteWindow />);
    await screen.findByRole("option", { name: /Start Deep 50/ });

    const input = screen.getByRole("textbox", { name: "Command" });
    fireEvent.change(input, { target: { value: "focus 50 on meridian" } });
    expect(screen.getAllByRole("option")[0]).toHaveTextContent("Start Deep 50 on “meridian”");
    fireEvent.keyDown(input, { key: "Enter" });

    await vi.waitFor(() => expect(m.timerApi.start).toHaveBeenCalledWith("deep", "meridian"));
    expect(m.appApi.hidePalette).toHaveBeenCalled();
  });

  it("moves with the arrow keys and opens settings", async () => {
    const m = await mocks;
    render(<PaletteWindow />);
    await screen.findByRole("option", { name: /Start Deep 50/ });

    const input = screen.getByRole("textbox", { name: "Command" });
    fireEvent.change(input, { target: { value: "settings" } });
    fireEvent.keyDown(input, { key: "Enter" });
    await vi.waitFor(() => expect(m.appApi.requestUi).toHaveBeenCalledWith("openSettings"));
  });

  it("closes on Escape", async () => {
    const m = await mocks;
    render(<PaletteWindow />);
    fireEvent.keyDown(screen.getByRole("textbox", { name: "Command" }), { key: "Escape" });
    expect(m.appApi.hidePalette).toHaveBeenCalled();
  });
});
