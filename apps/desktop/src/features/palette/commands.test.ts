import { describe, expect, it } from "vitest";
import type { TimerSnapshot } from "../focus/api";
import { filterItems, paletteItems, parseQuickStart } from "./commands";

const idle: TimerSnapshot = {
  phase: "focus",
  status: "idle",
  kind: "deep",
  plannedMs: 3_000_000,
  elapsedMs: 0,
  remainingMs: 3_000_000,
  overflowing: false,
  earnedBreakMs: 0,
  progressPermille: 0,
  completedInCycle: 0,
  longBreakEvery: 4,
  pausesLeft: 2,
  overflowEnabled: true,
  autoStartBreaks: true,
  intention: "",
};

describe("parseQuickStart", () => {
  it("reads a preset and an intention after 'on'", () => {
    expect(parseQuickStart("focus 50 on meridian")).toEqual({
      preset: "deep",
      intention: "meridian",
    });
  });

  it("reads a preset and the remaining words", () => {
    expect(parseQuickStart("25 review the PR")).toEqual({
      preset: "pomodoro",
      intention: "review the PR",
    });
    expect(parseQuickStart("flow refactor")).toEqual({
      preset: "flowtime",
      intention: "refactor",
    });
  });

  it("'focus' alone leaves the preset open", () => {
    expect(parseQuickStart("focus on writing")).toEqual({ preset: null, intention: "writing" });
  });

  it("ignores plain text", () => {
    expect(parseQuickStart("settings")).toBeNull();
    expect(parseQuickStart("   ")).toBeNull();
  });
});

describe("palette items", () => {
  it("offers presets when idle and controls when running", () => {
    expect(paletteItems(idle, false).map((i) => i.id)).toContain("start:deep");
    const running = paletteItems({ ...idle, status: "running" }, true).map((i) => i.label);
    expect(running).toEqual(
      expect.arrayContaining(["Pause", "Add 5 minutes", "End session", "Hide mini timer"]),
    );
    expect(running).not.toContain("Start Deep 50");
  });

  it("filters by words and ranks prefix matches first", () => {
    const items = paletteItems(idle, false);
    expect(filterItems(items, "mini", idle).map((i) => i.id)).toEqual(["mini"]);
    expect(filterItems(items, "log", idle)[0]?.id).toBe("log");
  });

  it("puts a quick start on top", () => {
    const [first] = filterItems(paletteItems(idle, false), "focus 50 on meridian", idle);
    expect(first?.label).toBe("Start Deep 50 on “meridian”");
    expect(first?.action).toEqual({ type: "start", preset: "deep", intention: "meridian" });
  });

  it("no quick start while a session runs", () => {
    const running = { ...idle, status: "running" as const };
    const items = filterItems(paletteItems(running, false), "focus 50 on x", running);
    expect(items.some((i) => i.id === "quick-start")).toBe(false);
  });
});
