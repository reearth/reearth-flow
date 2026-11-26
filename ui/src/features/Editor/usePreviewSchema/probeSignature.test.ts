import type { AnyWorkflowVariable } from "@flow/types";

import { probeSignature, referencedVariableNames } from "./probeSignature";

const variable = (
  name: string,
  defaultValue: unknown,
): AnyWorkflowVariable => ({
  id: name,
  name,
  defaultValue,
  type: "text",
  required: false,
  public: false,
});

describe("referencedVariableNames", () => {
  test("finds both accessor forms anywhere in the params", () => {
    expect(
      referencedVariableNames({
        dataset: { type: "flowExpr", value: 'variables["b"] + "/x"' },
        nested: [{ value: 'variables.get("a")' }],
      }),
    ).toEqual(["a", "b"]);
  });

  test("ignores look-alikes", () => {
    expect(referencedVariableNames({ v: 'myvariables["a"]' })).toEqual([]);
  });
});

describe("probeSignature", () => {
  const params = { dataset: { type: "flowExpr", value: 'variables["url"]' } };

  test("changes when a referenced variable's value changes", () => {
    expect(probeSignature(params, [variable("url", "a.csv")])).not.toBe(
      probeSignature(params, [variable("url", "b.csv")]),
    );
  });

  test("ignores variables the params do not reference", () => {
    expect(
      probeSignature(params, [variable("url", "a.csv"), variable("x", 1)]),
    ).toBe(
      probeSignature(params, [variable("url", "a.csv"), variable("x", 2)]),
    );
  });
});
