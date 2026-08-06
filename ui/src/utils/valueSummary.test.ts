import { describe, expect, test } from "vitest";

import {
  isLargeValue,
  safeSerialize,
  toSearchableString,
} from "./valueSummary";

describe("safeSerialize", () => {
  test("serializes small values whole so filtering can match inside them", () => {
    expect(safeSerialize({ a: 1, b: "two" })).toBe('{"a":1,"b":"two"}');
    expect(safeSerialize([1, 2, 3])).toBe("[1,2,3]");
    expect(safeSerialize("plain")).toBe('"plain"');
  });

  test("keeps the placeholders the table relies on", () => {
    expect(safeSerialize(undefined)).toBe("-");
    expect(safeSerialize(null)).toBe("null");
  });

  test("summarizes a large array instead of serializing it", () => {
    const result = safeSerialize(Array.from({ length: 50_000 }, (_, i) => i));

    expect(result).toContain("Array(50000)");
    expect(result.length).toBeLessThan(500);
  });

  test("does not choke on a cyclic value", () => {
    const cyclic: Record<string, unknown> = { name: "loop" };
    cyclic.self = cyclic;

    expect(() => safeSerialize(cyclic)).not.toThrow();
  });
});

describe("toSearchableString", () => {
  test("summarizes large values and serializes small ones", () => {
    expect(toSearchableString({ a: 1 })).toBe('{"a":1}');
    expect(toSearchableString(Array.from({ length: 5000 }, () => 0))).toContain(
      "Array(5000)",
    );
  });
});

describe("isLargeValue", () => {
  test("flags the bulky values a feature actually carries", () => {
    expect(isLargeValue(Array.from({ length: 101 }, () => 0))).toBe(true);
    expect(isLargeValue([1, 2, 3])).toBe(false);
  });
});
