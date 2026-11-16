import { describe, expect, it } from "vitest";

import type { AnyWorkflowVariable } from "@flow/types";

import {
  carryCustomizedValues,
  isCustomizedValue,
  toVariableDefaults,
} from "./runVariableDefaults";

const variable = (id: string, defaultValue: unknown): AnyWorkflowVariable => ({
  id,
  name: id,
  defaultValue,
  type: "choice",
  required: false,
  public: true,
});

describe("isCustomizedValue", () => {
  const runDefaults = toVariableDefaults([
    variable("mode", "fast"),
    variable("codes", ["a", "b"]),
  ]);

  it("is false for the default the run started from", () => {
    expect(isCustomizedValue("mode", "fast", runDefaults)).toBe(false);
    expect(isCustomizedValue("codes", ["a", "b"], runDefaults)).toBe(false);
  });

  it("is true for any other value", () => {
    expect(isCustomizedValue("mode", "slow", runDefaults)).toBe(true);
  });

  it("is true for a variable the run had no default for", () => {
    expect(isCustomizedValue("added", "x", runDefaults)).toBe(true);
  });
});

describe("carryCustomizedValues", () => {
  it("follows a default edited since the earlier run", () => {
    const previous = [variable("mode", "fast")];
    const previousDefaults = toVariableDefaults(previous);
    expect(
      carryCustomizedValues(
        [variable("mode", "slow")],
        previous,
        previousDefaults,
      ),
    ).toEqual([variable("mode", "slow")]);
  });

  it("keeps a value the user set for the earlier run", () => {
    expect(
      carryCustomizedValues(
        [variable("mode", "slow")],
        [variable("mode", "medium")],
        toVariableDefaults([variable("mode", "fast")]),
      ),
    ).toEqual([variable("mode", "medium")]);
  });

  it("keeps every earlier value when the earlier defaults are unknown", () => {
    expect(
      carryCustomizedValues(
        [variable("mode", "slow")],
        [variable("mode", "fast")],
      ),
    ).toEqual([variable("mode", "fast")]);
  });

  it("takes variables added since as they are", () => {
    expect(carryCustomizedValues([variable("added", "x")], [], {})).toEqual([
      variable("added", "x"),
    ]);
  });
});
