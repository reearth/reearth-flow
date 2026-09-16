import { describe, expect, it } from "vitest";

import { normalizeParams } from "./normalizeParams";

describe("normalizeParams", () => {
  it("returns undefined params untouched", () => {
    expect(normalizeParams(undefined)).toBeUndefined();
  });

  // `EngineReadyNode.with` is `any`, and a bare `with:` in an imported YAML
  // deserializes to null — none of these may reach `NodeData.params`, which is
  // a keyed record.
  it("does not leak non-record values into params", () => {
    expect(normalizeParams(null)).toBeUndefined();
    expect(normalizeParams("foo")).toBeUndefined();
    expect(normalizeParams(0)).toBeUndefined();
    expect(normalizeParams(42)).toBeUndefined();
    expect(normalizeParams(false)).toBeUndefined();
    expect(normalizeParams(["a", ""])).toBeUndefined();
  });

  it("drops empty strings and nullish values", () => {
    expect(
      normalizeParams({
        format: "geojson",
        prefix: "",
        suffix: null,
        comment: undefined,
      }),
    ).toEqual({ format: "geojson" });
  });

  it("keeps falsy values that are not empty", () => {
    expect(normalizeParams({ flatten: false, limit: 0 })).toEqual({
      flatten: false,
      limit: 0,
    });
  });

  it("keeps whitespace-only plain strings", () => {
    expect(normalizeParams({ delimiter: " " })).toEqual({ delimiter: " " });
  });

  it("drops empty code values but keeps filled ones", () => {
    expect(
      normalizeParams({
        calculation: { type: "flowExpr", value: "" },
        inline: { type: "string", value: "" },
        expr: { type: "flowExpr", value: "env.get('x')" },
      }),
    ).toEqual({ expr: { type: "flowExpr", value: "env.get('x')" } });
  });

  it("drops whitespace-only flowExpr values, but not whitespace-only strings", () => {
    expect(
      normalizeParams({
        expr: { type: "flowExpr", value: "  \n " },
        text: { type: "string", value: " " },
      }),
    ).toEqual({ text: { type: "string", value: " " } });
  });

  it("drops empty code values nested inside array rows", () => {
    expect(
      normalizeParams({
        aggregateAttributes: [
          {
            attribute: "we",
            attributeValue: { type: "flowExpr", value: "" },
            newAttribute: "2",
          },
        ],
      }),
    ).toEqual({
      aggregateAttributes: [{ attribute: "we", newAttribute: "2" }],
    });
  });

  it("drops array rows that have nothing left in them", () => {
    expect(
      normalizeParams({
        aggregateAttributes: [
          { attribute: "a", newAttribute: "b" },
          { attribute: "", attributeValue: { type: "flowExpr", value: "" } },
        ],
      }),
    ).toEqual({ aggregateAttributes: [{ attribute: "a", newAttribute: "b" }] });
  });

  it("drops empty strings inside arrays of strings", () => {
    expect(normalizeParams({ attributes: ["a", "", "b"] })).toEqual({
      attributes: ["a", "b"],
    });
  });

  it("drops containers left with no meaningful content", () => {
    expect(
      normalizeParams({
        method: "max",
        emptyList: [],
        blankList: [""],
        nested: { inner: { value: "" } },
      }),
    ).toEqual({ method: "max" });
  });

  it("normalizes deeply nested objects", () => {
    expect(
      normalizeParams({
        outer: {
          keep: "yes",
          drop: "",
          inner: { keep: 1, drop: { type: "flowExpr", value: "" } },
        },
      }),
    ).toEqual({ outer: { keep: "yes", inner: { keep: 1 } } });
  });

  it("strips the same params the engine-conversion step used to strip", () => {
    expect(
      normalizeParams({
        dataset: { value: "https://example.com/a.geojson", type: "string" },
        inline: { value: "", type: "string" },
        format: "geojson",
        prefix: "",
        nullish: null,
      }),
    ).toEqual({
      dataset: { value: "https://example.com/a.geojson", type: "string" },
      format: "geojson",
    });
  });

  it("returns an empty object when every param is empty", () => {
    expect(normalizeParams({ a: "", b: null })).toEqual({});
  });

  it("does not mutate the input", () => {
    const params = { keep: "a", drop: "" };
    normalizeParams(params);
    expect(params).toEqual({ keep: "a", drop: "" });
  });
});
