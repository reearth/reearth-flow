import { describe, expect, test } from "vitest";

import { findSelectedRow, selectionAfterRowClick } from "./hooks";

describe("findSelectedRow", () => {
  const station = { id: JSON.stringify("station"), _row: 0 };
  const rowsById = new Map<string, any>([["station", station]]);

  test("finds a row selected from the table, which stores the serialized id", () => {
    expect(findSelectedRow(rowsById, JSON.stringify("station"))).toBe(station);
  });

  test("finds a row selected from the map, which stores the parsed id", () => {
    expect(findSelectedRow(rowsById, "station")).toBe(station);
  });

  test("finds nothing for an id the rows do not hold", () => {
    expect(findSelectedRow(rowsById, "park")).toBeNull();
    expect(findSelectedRow(rowsById, JSON.stringify("park"))).toBeNull();
  });

  test("finds nothing without a selection or rows", () => {
    expect(findSelectedRow(rowsById, null)).toBeNull();
    expect(findSelectedRow(null, "station")).toBeNull();
  });
});

describe("selectionAfterRowClick", () => {
  const station = { id: JSON.stringify("station"), _row: 0 };
  const park = { id: JSON.stringify("park"), _row: 1 };
  const rowsById = new Map<string, any>([
    ["station", station],
    ["park", park],
  ]);

  test("selects the clicked row", () => {
    expect(selectionAfterRowClick(rowsById, null, station)).toBe(station.id);
  });

  test("moves the selection to another row", () => {
    expect(selectionAfterRowClick(rowsById, station.id, park)).toBe(park.id);
  });

  test("clears the selection when the selected row is clicked again", () => {
    expect(selectionAfterRowClick(rowsById, station.id, station)).toBeNull();
  });

  test("clears it whichever form the selection was stored in", () => {
    // A selection made from the map stores the parsed id.
    expect(selectionAfterRowClick(rowsById, "station", station)).toBeNull();
  });
});
