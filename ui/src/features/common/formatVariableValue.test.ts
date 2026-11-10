import { describe, expect, test } from "vitest";

import { formatVariableValue } from "./formatVariableValue";

describe("formatVariableValue", () => {
  test.each([
    ["text", "osaka", "osaka"],
    ["a number", 3, "3"],
    ["false", false, "false"],
    ["zero", 0, "0"],
    ["null", null, "N/A"],
    ["undefined", undefined, "N/A"],
    ["an empty string", "", "N/A"],
    ["an empty array", [], "N/A"],
    ["an array", ["a", 2], ["a", "2"]],
    ["an array of arrays", [["a"], { x: 1 }], ['["a"]', '{"x":1}']],
    ["an object", { x: 1 }, '{"x":1}'],
  ])("%s", (_, value, want) => {
    expect(formatVariableValue(value, "N/A")).toEqual(want);
  });
});
