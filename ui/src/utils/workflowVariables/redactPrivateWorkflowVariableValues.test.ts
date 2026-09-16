import type { AnyWorkflowVariable, VarType } from "@flow/types";

import { redactPrivateWorkflowVariableValues } from "./redactPrivateWorkflowVariableValues";

const variable = (
  name: string,
  isPublic: unknown,
  type: VarType = "text",
  defaultValue: any = `${name}-value`,
): AnyWorkflowVariable =>
  ({
    id: name,
    name,
    defaultValue,
    type,
    required: true,
    public: isPublic,
    config: { multiline: false },
  }) as AnyWorkflowVariable;

describe("redactPrivateWorkflowVariableValues", () => {
  test("keeps every variable, public or not", () => {
    const result = redactPrivateWorkflowVariableValues([
      variable("PUBLIC_REGION", true),
      variable("PRIVATE_KEY", false),
    ]);

    expect(result.map((v) => v.name)).toEqual(["PUBLIC_REGION", "PRIVATE_KEY"]);
  });

  test("leaves public values untouched", () => {
    const publicVar = variable("PUBLIC_REGION", true, "text", "ap-northeast-1");
    const [result] = redactPrivateWorkflowVariableValues([publicVar]);

    expect(result).toBe(publicVar);
    expect(result.defaultValue).toBe("ap-northeast-1");
  });

  test("resets a private value to the empty default for its type", () => {
    const result = redactPrivateWorkflowVariableValues([
      variable("SECRET_TEXT", false, "text", "hunter2"),
      variable("SECRET_COUNT", false, "number", 42),
      variable("SECRET_FLAG", false, "yes_no", true),
      variable("SECRET_LIST", false, "array", ["a", "b"]),
      variable("SECRET_PASSWORD", false, "password", "letmein"),
    ]);

    expect(result.map((v) => v.defaultValue)).toEqual(["", 0, false, [], ""]);
  });

  test("preserves everything about a private variable except its value", () => {
    const [result] = redactPrivateWorkflowVariableValues([
      variable("PRIVATE_KEY", false, "text", "hunter2"),
    ]);

    expect(result).toMatchObject({
      id: "PRIVATE_KEY",
      name: "PRIVATE_KEY",
      type: "text",
      required: true,
      public: false,
      config: { multiline: false },
    });
    expect(result.defaultValue).toBe("");
  });

  test("redacts when the public flag is missing or not exactly true", () => {
    const result = redactPrivateWorkflowVariableValues([
      variable("undefinedFlag", undefined, "text", "leak"),
      variable("nullFlag", null, "text", "leak"),
      variable("truthyString", "yes", "text", "leak"),
    ]);

    expect(result.every((v) => v.defaultValue === "")).toBe(true);
  });

  test("handles an absent list", () => {
    expect(redactPrivateWorkflowVariableValues(undefined)).toEqual([]);
    expect(redactPrivateWorkflowVariableValues([])).toEqual([]);
  });

  test("does not mutate the input", () => {
    const input = [variable("PRIVATE_KEY", false, "text", "hunter2")];
    redactPrivateWorkflowVariableValues(input);
    expect(input[0].defaultValue).toBe("hunter2");
  });
});
