import { formatDate } from "./date";

describe("formatDate", () => {
  // Helper function to set a specific timezone for testing
  const setTimezone = (timezone: string) => {
    const originalTimezone = process.env.TZ;
    process.env.TZ = timezone;
    return () => {
      process.env.TZ = originalTimezone;
    };
  };

  test("formats date correctly", () => {
    const date = new Date(2021, 0, 1, 9, 5); // Jan 1, 2021, 09:05
    expect(formatDate(date)).toBe("2021-1-1 9:05");
  });

  test("handles string input", () => {
    expect(formatDate("2021-01-01T09:05:00")).toBe("2021-1-1 9:05");
  });

  test("adds leading zero to minutes when less than 10", () => {
    const date = new Date(2021, 0, 1, 9, 5); // Jan 1, 2021, 09:05
    expect(formatDate(date)).toBe("2021-1-1 9:05");

    const dateWithSingleDigitMinute = new Date(2021, 0, 1, 9, 1); // Jan 1, 2021, 09:01
    expect(formatDate(dateWithSingleDigitMinute)).toBe("2021-1-1 9:01");
  });

  test("handles timezone differences", () => {
    const cleanup = setTimezone("UTC");
    const date = new Date(Date.UTC(2021, 0, 1, 0, 0)); // Jan 1, 2021, 00:00 UTC
    expect(formatDate(date)).toBe("2021-1-1 0:00");
    cleanup();
  });

  test("handles daylight saving time", () => {
    const cleanup = setTimezone("America/New_York");

    // During Standard Time
    const winterDate = new Date(2021, 0, 1, 0, 0); // Jan 1, 2021, 00:00
    expect(formatDate(winterDate)).toBe("2021-1-1 0:00");

    // During Daylight Saving Time
    const summerDate = new Date(2021, 6, 1, 0, 0); // Jul 1, 2021, 00:00
    expect(formatDate(summerDate)).toBe("2021-7-1 0:00");

    cleanup();
  });
});
