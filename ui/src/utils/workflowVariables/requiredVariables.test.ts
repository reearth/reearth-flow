import { describe, expect, test } from "vitest";

import type { AnyWorkflowVariable } from "@flow/types";

import {
  isEmptyVariableValue,
  missingRequiredVariables,
} from "./requiredVariables";

const def = (
  name: string,
  defaultValue: any,
  required = false,
): AnyWorkflowVariable => ({
  id: name,
  name,
  type: "text",
  defaultValue,
  required,
  public: true,
});

const definitions = [
  def("city", "Tokyo", true),
  def("apiKey", null, true),
  def("mode", "", false),
];

describe("isEmptyVariableValue", () => {
  test.each([[undefined], [null], [""], ["   "], [[]]])("%j is no value", (v) =>
    expect(isEmptyVariableValue(v)).toBe(true),
  );
  test.each([[0], [false], ["x"], [["a"]]])("%j is a value", (v) =>
    expect(isEmptyVariableValue(v)).toBe(false),
  );
});

describe("missingRequiredVariables", () => {
  test("names required variables with neither a value nor a default", () => {
    expect(missingRequiredVariables(definitions, undefined)).toEqual([
      "apiKey",
    ]);
  });

  test("a trigger value fills a required variable", () => {
    expect(missingRequiredVariables(definitions, { apiKey: "k" })).toEqual([]);
  });

  test("a cleared trigger value falls back to the default", () => {
    expect(missingRequiredVariables(definitions, { city: null })).toEqual([
      "apiKey",
    ]);
  });

  test("a blank trigger value does not", () => {
    expect(missingRequiredVariables(definitions, { city: " " })).toEqual([
      "city",
      "apiKey",
    ]);
  });
});

describe("missingRequiredVariables without overrides (debug run, manual deployment run)", () => {
  test("checks each variable's own value", () => {
    expect(
      missingRequiredVariables([
        def("a", "x", true),
        def("b", "", true),
        def("c", [], true),
        def("d", 0, true),
        def("e", false, true),
        def("f", undefined, true),
        def("g", "", false),
      ]),
    ).toEqual(["b", "c", "f"]);
  });
});
