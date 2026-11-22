import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { MainWindow } from "./MainWindow";

vi.mock("../../features/app/api", () => ({
  getAppInfo: vi.fn().mockResolvedValue({ version: "0.1.0", schemaVersion: 1 }),
}));

vi.mock("../../features/focus/FocusScreen", () => ({
  FocusScreen: () => <p>focus screen</p>,
}));

describe("MainWindow", () => {
  it("renders the sidebar and the focus screen", async () => {
    render(<MainWindow />);
    expect(screen.getByRole("navigation", { name: "Main" })).toBeInTheDocument();
    expect(screen.getByText("focus screen")).toBeInTheDocument();
    expect(await screen.findByText("v0.1.0")).toBeInTheDocument();
  });
});
