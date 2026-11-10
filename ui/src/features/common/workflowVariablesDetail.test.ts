import { describe, expect, test } from "vitest";

import { toWorkflowVariablesDetail } from "./workflowVariablesDetail";

const labels = { title: "Workflow Variables", empty: "N/A" };

describe("toWorkflowVariablesDetail", () => {
  test("adds no row when there are no variables", () => {
    expect(toWorkflowVariablesDetail([], labels)).toEqual([]);
    expect(toWorkflowVariablesDetail(undefined, labels)).toEqual([]);
  });

  test("one collapsible row holding each variable, in order", () => {
    expect(
      toWorkflowVariablesDetail(
        [
          { name: "area", value: "osaka" },
          { name: "ids", value: ["a", "b"] },
          { name: "note", value: null },
        ],
        labels,
      ),
    ).toEqual([
      {
        id: "workflowVariables",
        name: "Workflow Variables",
        value: "",
        type: "collapsible",
        items: [
          { id: "workflowVariables.area", name: "area", value: "osaka" },
          { id: "workflowVariables.ids", name: "ids", value: ["a", "b"] },
          { id: "workflowVariables.note", name: "note", value: "N/A" },
        ],
      },
    ]);
  });
});
