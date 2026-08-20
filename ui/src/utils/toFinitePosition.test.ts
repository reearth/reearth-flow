import { toFiniteNumber, toFinitePosition } from "./toFinitePosition";

describe("toFiniteNumber", () => {
  test("passes through a finite number", () => {
    expect(toFiniteNumber(429)).toBe(429);
    expect(toFiniteNumber(-200.25)).toBe(-200.25);
    expect(toFiniteNumber(0)).toBe(0);
  });

  test("replaces NaN with the fallback", () => {
    // This is the exact production gap: `NaN ?? 0` is NaN, so a nullish
    // fallback does not catch it. A persisted NaN position reaches ReactFlow
    // and triggers an infinite render loop (React #185), so the project cannot
    // be opened.
    expect(toFiniteNumber(NaN)).toBe(0);
  });

  test("replaces Infinity with the fallback", () => {
    expect(toFiniteNumber(Infinity)).toBe(0);
    expect(toFiniteNumber(-Infinity)).toBe(0);
  });

  test("replaces missing / non-numeric values with the fallback", () => {
    expect(toFiniteNumber(undefined)).toBe(0);
    expect(toFiniteNumber(null)).toBe(0);
    expect(toFiniteNumber("429")).toBe(0);
  });
});

describe("toFinitePosition", () => {
  test("passes through a valid position", () => {
    expect(toFinitePosition({ x: 429, y: 132 })).toEqual({ x: 429, y: 132 });
  });

  test("coerces a NaN position to the origin", () => {
    expect(toFinitePosition({ x: NaN, y: NaN })).toEqual({ x: 0, y: 0 });
  });

  test("coerces an empty / missing position to the origin", () => {
    expect(toFinitePosition({})).toEqual({ x: 0, y: 0 });
    expect(toFinitePosition(undefined)).toEqual({ x: 0, y: 0 });
  });

  test("fills only the non-finite axis", () => {
    expect(toFinitePosition({ x: 429, y: NaN })).toEqual({ x: 429, y: 0 });
  });
});
