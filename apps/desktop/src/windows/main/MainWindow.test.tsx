import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { MainWindow } from "./MainWindow";

vi.mock("../../features/app/api", () => ({
  getAppInfo: vi.fn().mockResolvedValue({ version: "0.1.0", schemaVersion: 0 }),
}));

describe("MainWindow", () => {
  it("renders and shows core info", async () => {
    render(<MainWindow />);
    expect(screen.getByRole("heading", { name: "Halo" })).toBeInTheDocument();
    expect(await screen.findByText("v0.1.0 · schema 0")).toBeInTheDocument();
  });
});
