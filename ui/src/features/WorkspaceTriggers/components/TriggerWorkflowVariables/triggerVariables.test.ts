import { describe, expect, test } from "vitest";

import type { AnyWorkflowVariable, TriggerVariableConfig } from "@flow/types";

import {
  isEmptyVariableValue,
  missingRequiredVariables,
  staleVariableNames,
  toTypedMappings,
} from "./triggerVariables";

const def = (
  name: string,
  type: AnyWorkflowVariable["type"],
  defaultValue: any,
  required = false,
  config?: AnyWorkflowVariable["config"],
): AnyWorkflowVariable => ({
  id: name,
  name,
  type,
  defaultValue,
  required,
  public: true,
  config,
});

const definitions = [
  def("city", "text", "Tokyo", true),
  def("apiKey", "text", null, true),
  def("mode", "choice", "fast", false, { choices: ["fast", "slow"] } as any),
];

const inferred = (name: string, value: any): TriggerVariableConfig => ({
  name,
  type: "text",
  defaultValue: value,
  deploymentDefault: undefined,
});

describe("isEmptyVariableValue", () => {
  test.each([[undefined], [null], [""], ["   "], [[]]])("%j is no value", (v) =>
    expect(isEmptyVariableValue(v)).toBe(true),
  );
  test.each([[0], [false], ["x"], [["a"]]])("%j is a value", (v) =>
    expect(isEmptyVariableValue(v)).toBe(false),
  );
});

describe("toTypedMappings", () => {
  test("takes each variable's type, config and required from the deployment", () => {
    const rows = toTypedMappings(definitions, {}, inferred);
    expect(rows.map((r) => [r.name, r.type, r.required])).toEqual([
      ["city", "text", true],
      ["apiKey", "text", true],
      ["mode", "choice", false],
    ]);
    expect(rows[2].config).toEqual({ choices: ["fast", "slow"] });
  });

  test("uses the trigger's value over the deployment default", () => {
    const [city] = toTypedMappings(definitions, { city: "Osaka" }, inferred);
    expect(city.defaultValue).toBe("Osaka");
    expect(city.deploymentDefault).toBe("Tokyo");
  });

  test("keeps a value for a variable the deployment dropped, marked stale", () => {
    const rows = toTypedMappings(definitions, { oldVar: 1 }, inferred);
    expect(rows.at(-1)).toEqual(
      expect.objectContaining({ name: "oldVar", stale: true }),
    );
  });
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

describe("staleVariableNames", () => {
  test("names trigger values the deployment has no variable for", () => {
    expect(staleVariableNames(definitions, { city: "x", oldVar: 1 })).toEqual([
      "oldVar",
    ]);
  });
});
