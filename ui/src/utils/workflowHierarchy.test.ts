import { DEFAULT_ENTRY_GRAPH_ID as MAIN } from "@flow/global-constants";
import type { Workflow } from "@flow/types";

import {
  buildWorkflowParentMap,
  getWorkflowLineage,
  isWorkflowDescendant,
} from "./workflowHierarchy";

const workflow = (id: string, subworkflowIds: string[] = []): Workflow => ({
  id,
  name: id,
  nodes: subworkflowIds.map((subworkflowId) => ({
    id: `node-${subworkflowId}`,
    type: "subworkflow",
    position: { x: 0, y: 0 },
    data: { officialName: "Subworkflow", subworkflowId },
  })),
  edges: [],
});

// main → a → b, main → c
const workflows = [
  workflow(MAIN, ["a", "c"]),
  workflow("a", ["b"]),
  workflow("b"),
  workflow("c"),
];

describe("workflowHierarchy", () => {
  const parentMap = buildWorkflowParentMap(workflows);

  it("maps each subworkflow to the workflow that references it", () => {
    expect(parentMap).toEqual({ a: MAIN, b: "a", c: MAIN });
  });

  it("returns the lineage from main down to the workflow", () => {
    expect(getWorkflowLineage(parentMap, "b")).toEqual([MAIN, "a", "b"]);
    expect(getWorkflowLineage(parentMap, MAIN)).toEqual([MAIN]);
  });

  it("hangs an unreferenced workflow off main", () => {
    expect(getWorkflowLineage(parentMap, "orphan")).toEqual([MAIN, "orphan"]);
  });

  it("stops on a reference cycle", () => {
    expect(getWorkflowLineage({ x: "y", y: "x" }, "x")).toEqual(["y", "x"]);
  });

  it("detects descendants", () => {
    expect(isWorkflowDescendant(parentMap, "b", "a")).toBe(true);
    expect(isWorkflowDescendant(parentMap, "b", MAIN)).toBe(true);
    expect(isWorkflowDescendant(parentMap, "c", "a")).toBe(false);
    expect(isWorkflowDescendant(parentMap, "a", "a")).toBe(false);
  });
});
