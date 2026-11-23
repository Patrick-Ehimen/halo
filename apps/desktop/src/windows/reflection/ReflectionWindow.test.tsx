import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ReflectionWindow } from "./ReflectionWindow";

const api = vi.hoisted(() => ({
  statsApi: {
    view: vi.fn(),
    exportSessions: vi.fn(),
    reflection: vi.fn(),
    reflect: vi.fn(),
  },
}));
vi.mock("../../features/stats/api", () => api);
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ onFocusChanged: () => Promise.resolve(() => {}) }),
}));

const view = {
  focusedLabel: "50:00",
  intention: "Write matching-engine tests",
  distractions: 1,
  notes: "",
  breakLabel: "Start 10 min break",
};

describe("ReflectionWindow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.statsApi.reflect.mockResolvedValue(null);
  });

  it("rates the session, counts distractions, and starts the break", async () => {
    api.statsApi.reflection.mockResolvedValue(view);
    render(<ReflectionWindow />);

    expect(await screen.findByText("Session complete · 50:00")).toBeInTheDocument();
    expect(screen.getByText("Write matching-engine tests")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "In flow" }));
    expect(screen.getByRole("button", { name: "In flow" })).toHaveAttribute("aria-pressed", "true");
    fireEvent.click(screen.getByRole("button", { name: "More" }));
    fireEvent.change(screen.getByLabelText("Session notes"), {
      target: { value: "Fill engine passes" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Start 10 min break" }));
    expect(api.statsApi.reflect).toHaveBeenCalledWith({
      rating: 4,
      distractions: 2,
      notes: "Fill engine passes",
      startBreak: true,
    });
  });

  it("offers Done when no break is waiting", async () => {
    api.statsApi.reflection.mockResolvedValue({ ...view, breakLabel: null });
    render(<ReflectionWindow />);
    await screen.findByText("Session complete · 50:00");
    expect(screen.queryByRole("button", { name: /break/ })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Done" }));
    expect(api.statsApi.reflect).toHaveBeenCalledWith({
      rating: null,
      distractions: 1,
      notes: "",
      startBreak: false,
    });
  });
});
