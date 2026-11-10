import { describe, expect, test } from "vitest";

import type { AnyWorkflowVariable } from "@flow/types";

import { toRunOverrides } from "./runOverrides";

const variable = (
  name: string,
  type: AnyWorkflowVariable["type"],
  defaultValue: any,
): AnyWorkflowVariable =>
  ({ id: name, name, type, defaultValue }) as AnyWorkflowVariable;

describe("toRunOverrides", () => {
  const defaults = [
    variable("area", "text", "tokyo"),
    variable("ids", "array", ["a"]),
    variable("note", "text", undefined),
  ];

  test("sends nothing when no value changed", () => {
    expect(toRunOverrides(defaults, defaults)).toEqual([]);
  });

  test("sends only the values that changed, by name", () => {
    expect(
      toRunOverrides(defaults, [
        variable("area", "text", "osaka"),
        defaults[1],
        defaults[2],
      ]),
    ).toEqual([{ key: "area", type: "text", value: "osaka" }]);
  });

  test("compares arrays by their items", () => {
    expect(
      toRunOverrides(defaults, [
        defaults[0],
        variable("ids", "array", ["a"]),
        defaults[2],
      ]),
    ).toEqual([]);
    expect(
      toRunOverrides(defaults, [
        defaults[0],
        variable("ids", "array", ["a", "b"]),
        defaults[2],
      ]),
    ).toEqual([{ key: "ids", type: "array", value: ["a", "b"] }]);
  });

  test("sends a cleared value as null", () => {
    expect(
      toRunOverrides(defaults, [
        variable("area", "text", undefined),
        defaults[1],
        defaults[2],
      ]),
    ).toEqual([{ key: "area", type: "text", value: null }]);
  });

  test("treats an unset default and null as the same", () => {
    expect(
      toRunOverrides(defaults, [
        defaults[0],
        defaults[1],
        variable("note", "text", null),
      ]),
    ).toEqual([]);
  });
});
