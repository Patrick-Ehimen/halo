import { describe, expect, it } from "vitest";
import { lanes } from "./lanes";

const at = (id: string, start: number, end: number) => ({ id, start, end });
const layout = (items: ReturnType<typeof at>[]) =>
  Object.fromEntries(lanes(items).map((l) => [l.item.id, [l.lane, l.of]]));

describe("lanes", () => {
  it("gives items that don't overlap the full width", () => {
    expect(layout([at("a", 900, 950), at("b", 960, 1020)])).toEqual({ a: [0, 1], b: [0, 1] });
  });

  it("puts back-to-back items in one column", () => {
    expect(layout([at("a", 900, 960), at("b", 960, 1020)])).toEqual({ a: [0, 1], b: [0, 1] });
  });

  it("splits overlapping items into columns and reuses a free one", () => {
    // 15:00–16:00, 15:30–17:00, 16:00–17:00: the third fits under the first.
    expect(layout([at("a", 900, 960), at("b", 930, 1020), at("c", 960, 1020)])).toEqual({
      a: [0, 2],
      b: [1, 2],
      c: [0, 2],
    });
  });

  it("keeps a zero-length item from being covered", () => {
    expect(layout([at("a", 900, 900), at("b", 910, 960)])).toEqual({ a: [0, 2], b: [1, 2] });
  });
});
