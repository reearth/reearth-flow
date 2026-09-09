import { describe, expect, test } from "vitest";
import * as Y from "yjs";

import type { Node } from "@flow/types";

import { rebuildWorkflow } from "./rebuildWorkflow";
import { updateYNode } from "./yUpdaters";
import { yWorkflowConstructor } from "./yWorkflowConstructor";

import type { YNodesMap, YWorkflow } from "../types";

const batchNode = (style?: { width: string; height: string }): Node => ({
  id: "batch-1",
  type: "batch",
  position: { x: 0, y: 0 },
  data: { officialName: "Batch" },
  ...(style ? { style } : {}),
});

describe("updateYNode", () => {
  test("a node that loses its style stays readable", () => {
    // Emptying the nested map in place rather than removing it left a `style`
    // entry present but without its keys, which threw on the next read and
    // surfaced as a project-wide corruption error.
    const doc = new Y.Doc();
    const yWorkflows = doc.getMap<YWorkflow>("workflows");
    yWorkflows.set(
      "main",
      yWorkflowConstructor("main", "Main", [
        batchNode({ width: "300px", height: "200px" }),
      ]),
    );

    const yWorkflow = yWorkflows.get("main") as YWorkflow;
    const yNodes = yWorkflow.get("nodes") as YNodesMap;
    const yNode = yNodes.get("batch-1");
    if (!yNode) throw new Error("fixture missing");

    updateYNode(yNode, batchNode());

    expect(() => rebuildWorkflow(yWorkflow)).not.toThrow();
    const rebuilt = (rebuildWorkflow(yWorkflow).nodes as Node[])[0];
    expect(rebuilt.id).toBe("batch-1");
    expect(rebuilt.style).toBeUndefined();
  });

  test("style is still updated in place when values are present", () => {
    const doc = new Y.Doc();
    const yWorkflows = doc.getMap<YWorkflow>("workflows");
    yWorkflows.set(
      "main",
      yWorkflowConstructor("main", "Main", [
        batchNode({ width: "300px", height: "200px" }),
      ]),
    );

    const yWorkflow = yWorkflows.get("main") as YWorkflow;
    const yNodes = yWorkflow.get("nodes") as YNodesMap;
    const yNode = yNodes.get("batch-1");
    if (!yNode) throw new Error("fixture missing");

    const styleMapBefore = yNode.get("style");
    updateYNode(yNode, batchNode({ width: "500px", height: "400px" }));

    // Same Y.Map instance - mutated, not replaced.
    expect(yNode.get("style")).toBe(styleMapBefore);
    const rebuilt = (rebuildWorkflow(yWorkflow).nodes as Node[])[0];
    expect(rebuilt.style).toEqual({ width: "500px", height: "400px" });
  });
});
