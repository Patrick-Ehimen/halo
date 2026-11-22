import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useFocusStore } from "../../features/focus/store";
import { CheckInWindow } from "./CheckInWindow";

const mocks = vi.hoisted(async () => (await import("../../test/timerMocks")).makeMocks());

vi.mock("../../features/focus/api", async () => {
  const m = await mocks;
  return { timerApi: m.timerApi, subscribeTimer: m.subscribeTimer };
});
vi.mock("../../features/app/api", async () => ({ appApi: (await mocks).appApi }));

describe("CheckInWindow", () => {
  beforeEach(async () => {
    const { running } = await import("../../test/timerMocks");
    useFocusStore.setState({
      snapshot: null,
      today: null,
      settings: null,
      prefs: null,
      error: null,
    });
    (await mocks).timerApi.state.mockResolvedValue(running);
  });

  it("asks and records the answer", async () => {
    const m = await mocks;
    m.timerApi.checkIn.mockResolvedValue(null);
    render(<CheckInWindow />);

    expect(screen.getByText("Still on it?")).toBeInTheDocument();
    expect(await screen.findByText("Write matching-engine tests")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "I drifted" }));
    expect(m.timerApi.checkIn).toHaveBeenCalledWith("drifted");
    fireEvent.click(screen.getByRole("button", { name: "Yes, focused" }));
    expect(m.timerApi.checkIn).toHaveBeenCalledWith("focused");
  });
});
