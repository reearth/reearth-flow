import { describe, expect, it } from "vitest";

import type { AnyWorkflowVariable } from "@flow/types";

import { toDebugRunVariables } from "./debugRunVariables";

const variable = (
  name: string,
  defaultValue: unknown,
  id = `id-${name}`,
): AnyWorkflowVariable => ({
  id,
  name,
  defaultValue,
  type: "text",
  required: false,
  public: true,
});

describe("toDebugRunVariables", () => {
  it("prefers the values recorded on the job over the local copy", () => {
    expect(
      toDebugRunVariables({
        recorded: [{ key: "city", type: "text", value: "Tokyo" }],
        local: [variable("city", "Osaka")],
      }),
    ).toEqual([{ key: "0-city", name: "city", value: "Tokyo" }]);
  });

  it("falls back to the local copy when the job recorded none", () => {
    expect(
      toDebugRunVariables({
        recorded: [],
        local: [variable("city", "Osaka")],
      }),
    ).toEqual([{ key: "id-city", name: "city", value: "Osaka" }]);
  });

  it("compares recorded values with the run's defaults via the local copy", () => {
    // The run used the default "fast"; the project default became "slow" later.
    expect(
      toDebugRunVariables({
        recorded: [{ key: "mode", type: "choice", value: "fast" }],
        local: [variable("mode", "fast")],
        runDefaults: { "id-mode": "fast" },
        defaults: [variable("mode", "slow")],
      }),
    ).toEqual([{ key: "0-mode", name: "mode", value: "fast" }]);
  });

  it("does not mark recorded values whose name is shared", () => {
    expect(
      toDebugRunVariables({
        recorded: [{ key: "Same", type: "text", value: "Option 2" }],
        local: [variable("Same", "Option 2", "a"), variable("Same", "x", "b")],
        runDefaults: { a: "Option 2", b: "hello" },
      }),
    ).toEqual([{ key: "0-Same", name: "Same", value: "Option 2" }]);
  });

  it("marks recorded values that differ from the project's default", () => {
    expect(
      toDebugRunVariables({
        recorded: [
          { key: "city", type: "text", value: "Tokyo" },
          { key: "zoom", type: "number", value: 3 },
        ],
        defaults: [variable("city", "Osaka"), variable("zoom", 3)],
      }),
    ).toEqual([
      {
        key: "0-city",
        name: "city",
        value: "Tokyo",
        changedFrom: { defaultValue: "Osaka" },
      },
      { key: "1-zoom", name: "zoom", value: 3 },
    ]);
  });

  it("compares structured values by content", () => {
    expect(
      toDebugRunVariables({
        local: [variable("codes", ["a", "b"])],
        runDefaults: { "id-codes": ["a", "b"] },
      }),
    ).toEqual([{ key: "id-codes", name: "codes", value: ["a", "b"] }]);
  });

  it("does not mark a variable the project no longer has", () => {
    expect(
      toDebugRunVariables({
        recorded: [{ key: "removed", type: "text", value: "x" }],
        defaults: [variable("city", "Osaka")],
      }),
    ).toEqual([{ key: "0-removed", name: "removed", value: "x" }]);
  });

  it("matches variables that share a name by id", () => {
    const first = variable("Same", "Option 2", "a");
    const second = variable("Same", "hello", "b");
    expect(
      toDebugRunVariables({
        local: [first, { ...second, defaultValue: "changed" }],
        runDefaults: { a: "Option 2", b: "hello" },
        defaults: [first, second],
      }),
    ).toEqual([
      { key: "a", name: "Same", value: "Option 2" },
      {
        key: "b",
        name: "Same",
        value: "changed",
        changedFrom: { defaultValue: "hello" },
      },
    ]);
  });

  it("does not guess between recorded variables that share a name", () => {
    expect(
      toDebugRunVariables({
        recorded: [{ key: "Same", type: "text", value: "x" }],
        defaults: [variable("Same", "a", "a"), variable("Same", "b", "b")],
      }),
    ).toEqual([{ key: "0-Same", name: "Same", value: "x" }]);
  });

  it("compares with the defaults the run started from", () => {
    // The run used the default "fast"; the project default became "slow" later.
    expect(
      toDebugRunVariables({
        local: [variable("mode", "fast")],
        runDefaults: { "id-mode": "fast" },
        defaults: [variable("mode", "slow")],
      }),
    ).toEqual([{ key: "id-mode", name: "mode", value: "fast" }]);
  });

  it("falls back to the current default by id for runs saved without defaults", () => {
    expect(
      toDebugRunVariables({
        local: [variable("mode", "medium")],
        defaults: [variable("mode", "fast")],
      }),
    ).toEqual([
      {
        key: "id-mode",
        name: "mode",
        value: "medium",
        changedFrom: { defaultValue: "fast" },
      },
    ]);
  });

  it("leaves out variables without a value", () => {
    expect(
      toDebugRunVariables({
        local: [variable("city", null), variable("zoom", undefined)],
      }),
    ).toEqual([]);
  });
});
