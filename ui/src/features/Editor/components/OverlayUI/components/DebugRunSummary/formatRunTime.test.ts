import { describe, expect, it } from "vitest";

import { formatRunTime } from "./formatRunTime";

// Local times, so the expectations hold in any time zone.
const now = new Date(2026, 10, 10, 15, 0);

describe("formatRunTime", () => {
  it("shows only the time for a run from today", () => {
    expect(
      formatRunTime(new Date(2026, 10, 10, 9, 5).toISOString(), "en-GB", now),
    ).toBe("09:05");
  });

  it("adds the date for a run from another day", () => {
    expect(
      formatRunTime(new Date(2026, 10, 9, 18, 2).toISOString(), "en-GB", now),
    ).toBe("9 Nov, 18:02");
  });

  it("adds the year for a run from another year", () => {
    expect(
      formatRunTime(new Date(2025, 11, 31, 8, 0).toISOString(), "en-GB", now),
    ).toBe("31 Dec 2025, 08:00");
  });

  it("returns nothing for a missing or unreadable time", () => {
    expect(formatRunTime(undefined, "en-GB", now)).toBeUndefined();
    expect(formatRunTime("not a date", "en-GB", now)).toBeUndefined();
  });

  it("falls back to the browser's locale for an unknown tag", () => {
    expect(
      formatRunTime(
        new Date(2026, 10, 10, 9, 5).toISOString(),
        "und-xx-?",
        now,
      ),
    ).toBeDefined();
  });
});
