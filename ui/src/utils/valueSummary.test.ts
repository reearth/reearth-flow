import { describe, expect, test } from "vitest";

import {
  formatStructured,
  isLargeValue,
  previewSerialize,
  safeSerialize,
  toSearchableString,
} from "./valueSummary";

/** A polygon ring of `count` positions, as the transform emits them. */
const ring = (count: number) =>
  Array.from({ length: count }, (_, i) => [139.7 + i / 1000, 35.6, 10]);

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

  test("previews a large array instead of serializing it", () => {
    const result = safeSerialize(Array.from({ length: 50_000 }, (_, i) => i));

    expect(result).toContain("… +49,");
    expect(result.length).toBeLessThan(600);
  });

  test("a large coordinate list still reads as coordinates", () => {
    // What the table shows for a polygon: one ring, far past the threshold.
    const result = safeSerialize([ring(500)]);

    // The cell renderer only shows the first ~100 characters, so the numbers
    // have to be at the front — not a count and a shape.
    expect(result.slice(0, 100)).toContain("[139.7, 35.6, 10]");
    expect(result.slice(0, 100)).not.toContain("Array(");
    expect(result).toContain("… +");
    expect(result.length).toBeLessThan(600);
  });

  test("does not choke on a cyclic value", () => {
    const cyclic: Record<string, unknown> = { name: "loop" };
    cyclic.self = cyclic;

    expect(() => safeSerialize(cyclic)).not.toThrow();
    expect(() => previewSerialize(cyclic)).not.toThrow();
    expect(() => formatStructured(cyclic)).not.toThrow();
  });
});

describe("formatStructured", () => {
  test("keeps a coordinate on one line and opens a block for the rings", () => {
    const result = formatStructured([ring(3)]);

    expect(result).toBe(
      [
        "[",
        "  [",
        "    [139.7, 35.6, 10],",
        "    [139.701, 35.6, 10],",
        "    [139.702, 35.6, 10]",
        "  ]",
        "]",
      ].join("\n"),
    );
  });

  test("elides a long list with the count it cut, rather than printing it all", () => {
    const result = formatStructured([ring(500)]);

    expect(result).toContain("[139.7, 35.6, 10],");
    expect(result).toContain("… +460");
    expect(result.split("\n").length).toBeLessThan(60);
  });

  test("stays bounded on a mesh far larger than the block budget", () => {
    const faces = Array.from({ length: 2_000 }, () => [ring(200)]);

    expect(formatStructured(faces).length).toBeLessThan(30_000);
  });

  test("says how much it cut without a word to translate", () => {
    // The cut marker used to read "… 460 more". That string is written onto a
    // table row by `safeSerialize` at load time, not rendered through a
    // component, so an English word there survives a language change and shows
    // up untranslated in a Japanese UI. Digits and symbols read the same
    // everywhere.
    const result = formatStructured(Array.from({ length: 5_000 }, (_, i) => i));

    expect(result).toContain("…");
    expect(result).toMatch(/^[\d\s,.\u2026+[\]-]+$/);
  });

  test("names a shape it stopped at without a word either", () => {
    // Twelve wrappers puts the innermost object at MAX_DEPTH, where the
    // printer stops descending and reports its shape instead.
    let nested: unknown = { a: 1, b: 2, c: 3 };
    for (let i = 0; i < 12; i++) nested = { down: nested };

    const result = formatStructured(nested);
    expect(result).toContain("Object(3)");
    expect(result).not.toContain("keys");
  });

  test("renders scalars and empty containers plainly", () => {
    expect(formatStructured("Point")).toBe('"Point"');
    expect(formatStructured(6697)).toBe("6697");
    expect(formatStructured(null)).toBe("null");
    expect(formatStructured([])).toBe("[]");
    expect(formatStructured({})).toBe("{}");
  });
});

describe("toSearchableString", () => {
  test("previews large values and serializes small ones", () => {
    expect(toSearchableString({ a: 1 })).toBe('{"a":1}');

    const large = toSearchableString(Array.from({ length: 5000 }, () => 0));
    expect(large).toContain("0, 0, 0");
    expect(large.length).toBeLessThan(600);
  });
});

describe("isLargeValue", () => {
  test("flags the bulky values a feature actually carries", () => {
    expect(isLargeValue(Array.from({ length: 101 }, () => 0))).toBe(true);
    expect(isLargeValue([1, 2, 3])).toBe(false);
  });
});
