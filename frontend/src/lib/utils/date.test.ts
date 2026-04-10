import { describe, it, expect } from "vitest";
import { isThisWeek, isToday } from "./date";

// Fixed reference: Wednesday 2026-04-08 12:00
const now = new Date("2026-04-08T12:00:00");

describe("isThisWeek", () => {
  it("returns true for Monday of the current week", () => {
    expect(isThisWeek("2026-04-06T10:00:00", now)).toBe(true);
  });

  it("returns true for Sunday of the current week", () => {
    expect(isThisWeek("2026-04-12T20:00:00", now)).toBe(true);
  });

  it("returns true for the same day", () => {
    expect(isThisWeek("2026-04-08T15:00:00", now)).toBe(true);
  });

  it("returns false for last Sunday", () => {
    expect(isThisWeek("2026-04-05T23:59:59", now)).toBe(false);
  });

  it("returns false for next Monday", () => {
    expect(isThisWeek("2026-04-13T00:00:00", now)).toBe(false);
  });

  it("returns false for a date weeks away", () => {
    expect(isThisWeek("2026-05-01T10:00:00", now)).toBe(false);
  });

  it("handles Sunday as now correctly", () => {
    const sunday = new Date("2026-04-12T23:00:00");
    // Monday of that week is still April 6
    expect(isThisWeek("2026-04-06T10:00:00", sunday)).toBe(true);
    expect(isThisWeek("2026-04-12T23:59:00", sunday)).toBe(true);
    expect(isThisWeek("2026-04-13T00:00:00", sunday)).toBe(false);
  });
});

describe("isToday", () => {
  it("returns true for a date today", () => {
    expect(isToday("2026-04-08T20:00:00", now)).toBe(true);
  });

  it("returns false for tomorrow", () => {
    expect(isToday("2026-04-09T10:00:00", now)).toBe(false);
  });

  it("returns false for yesterday", () => {
    expect(isToday("2026-04-07T10:00:00", now)).toBe(false);
  });
});
