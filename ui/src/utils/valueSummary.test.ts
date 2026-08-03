import { afterEach, describe, expect, test } from "vitest";

import {
  clearRasterStore,
  extractAppearance,
} from "@flow/lib/intermediateData";

import {
  isLargeValue,
  safeSerialize,
  toSearchableString,
} from "./valueSummary";

afterEach(() => clearRasterStore());

/** Strip a texture out of a feature and hand back the handle left behind. */
function handleFor(byteLength: number) {
  const geometry = {
    Euclidean3D: {
      Polygon: {
        frame: { Crs: 4979 },
        exterior: [],
        appearance: {
          materials: [
            {
              Pbr: {
                base_color_map: {
                  raster: {
                    InMemory: {
                      mime_type: "image/png",
                      bytes: Array.from(
                        { length: byteLength },
                        (_, i) => i % 256,
                      ),
                    },
                  },
                },
              },
            },
          ],
          themes: [],
          default_theme: "default",
        },
      },
    },
  };
  extractAppearance(geometry, "owner");
  return geometry.Euclidean3D.Polygon.appearance.materials[0].Pbr.base_color_map
    .raster.InMemory;
}

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

  test("describes an image rather than reserializing its bytes", () => {
    const result = safeSerialize(handleFor(2048));

    expect(result).toBe("image/png · 2.00 KB");
  });

  test("does not choke on a cyclic value", () => {
    const cyclic: Record<string, unknown> = { name: "loop" };
    cyclic.self = cyclic;

    expect(() => safeSerialize(cyclic)).not.toThrow();
  });
});

describe("toSearchableString", () => {
  test("makes an image searchable by type rather than by pixel", () => {
    expect(toSearchableString(handleFor(1024))).toBe("image/png · 1.00 KB");
  });

  test("summarizes large values and serializes small ones", () => {
    expect(toSearchableString({ a: 1 })).toBe('{"a":1}');
    expect(toSearchableString(Array.from({ length: 5000 }, () => 0))).toContain(
      "Array(5000)",
    );
  });
});

describe("isLargeValue", () => {
  test("treats an image handle as small, since it no longer holds pixels", () => {
    expect(isLargeValue(handleFor(1_000_000))).toBe(false);
  });

  test("flags the bulky values a feature actually carries", () => {
    expect(isLargeValue(Array.from({ length: 101 }, () => 0))).toBe(true);
    expect(isLargeValue([1, 2, 3])).toBe(false);
  });
});
