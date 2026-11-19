import { describe, expect, test } from "vitest";

import { cellValue, toTableRow } from "./useFeatureRows";

describe("table rows", () => {
  test("cells are the attributes as JSON, missing ones as null", () => {
    const row = toTableRow({
      row: 1,
      id: "b",
      attributes: { name: "Shinjuku" },
    });
    expect(cellValue(row, "name")).toBe('"Shinjuku"');
    expect(cellValue(row, "height")).toBe("null");
  });

  test("the id is serialized, falling back to the row's line", () => {
    expect(toTableRow({ row: 0, id: "a", attributes: {} }).id).toBe('"a"');
    expect(toTableRow({ row: 3, id: undefined, attributes: {} }).id).toBe("3");
  });
});
