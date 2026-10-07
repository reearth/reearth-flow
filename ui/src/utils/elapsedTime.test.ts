import { formatElapsedTime } from "./elapsedTime";

describe("formatElapsedTime", () => {
  test("formats durations under an hour as m:ss", () => {
    expect(formatElapsedTime(0)).toBe("0:00");
    expect(formatElapsedTime(5_000)).toBe("0:05");
    expect(formatElapsedTime(65_000)).toBe("1:05");
    expect(formatElapsedTime(59 * 60_000 + 59_000)).toBe("59:59");
  });

  test("formats durations of an hour or more as h:mm:ss", () => {
    expect(formatElapsedTime(3_600_000)).toBe("1:00:00");
    expect(formatElapsedTime(3_600_000 + 5 * 60_000 + 7_000)).toBe("1:05:07");
  });

  test("drops partial seconds", () => {
    expect(formatElapsedTime(1_999)).toBe("0:01");
  });

  test("clamps negative durations to zero", () => {
    expect(formatElapsedTime(-4_000)).toBe("0:00");
  });
});
