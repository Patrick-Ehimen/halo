import { describe, expect, it } from "vitest";
import { parseTime } from "./time";

describe("parseTime", () => {
  it("reads 24-hour times typed in different ways", () => {
    expect(parseTime("15:30")).toBe("15:30");
    expect(parseTime("1530")).toBe("15:30");
    expect(parseTime("9:05")).toBe("09:05");
    expect(parseTime("905")).toBe("09:05");
    expect(parseTime("9")).toBe("09:00");
    expect(parseTime(" 9.30 ")).toBe("09:30");
    expect(parseTime("")).toBe("");
  });

  it("rejects what isn't a time", () => {
    expect(parseTime("24:00")).toBeNull();
    expect(parseTime("12:60")).toBeNull();
    expect(parseTime("noon")).toBeNull();
    expect(parseTime("3pm")).toBeNull();
  });
});
