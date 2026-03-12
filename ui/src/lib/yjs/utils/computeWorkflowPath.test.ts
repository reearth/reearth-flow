import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import type { Node, Workflow } from "@flow/types";

import { computeWorkflowPath } from "./computeWorkflowPath";

const createNode = (
  id: string,
  type: "subworkflow" | "transformer" = "transformer",
  subworkflowId?: string,
): Node => ({
  id,
  type,
  position: { x: 0, y: 0 },
  data: {
    officialName: `Node ${id}`,
    subworkflowId,
  },
});

const createWorkflow = (
  id: string,
  nodes: Node[] = [],
  name?: string,
): Workflow => ({
  id,
  name: name ?? `Workflow ${id}`,
  nodes,
  edges: [],
});

describe("computeWorkflowPath", () => {
  describe("when currentWorkflowId is undefined or empty", () => {
    it("should return empty string when currentWorkflowId is undefined", () => {
      const workflows: Workflow[] = [createWorkflow("main")];

      const result = computeWorkflowPath(workflows, undefined);

      expect(result).toBe("");
    });

    it("should return empty string when currentWorkflowId is empty string", () => {
      const workflows: Workflow[] = [createWorkflow("main")];

      const result = computeWorkflowPath(workflows, "");

      expect(result).toBe("");
    });
  });

  describe("when currentWorkflowId is DEFAULT_ENTRY_GRAPH_ID", () => {
    it("should return empty string for main workflow", () => {
      const workflows: Workflow[] = [createWorkflow(DEFAULT_ENTRY_GRAPH_ID)];

      const result = computeWorkflowPath(workflows, DEFAULT_ENTRY_GRAPH_ID);

      expect(result).toBe("");
    });
  });

  describe("single-level nesting (parent-child)", () => {
    it("should return just the child workflow id for direct child of main", () => {
      const childWorkflowId = "subworkflow-1";
      const mainWorkflow = createWorkflow(DEFAULT_ENTRY_GRAPH_ID, [
        createNode("node-1", "subworkflow", childWorkflowId),
      ]);
      const childWorkflow = createWorkflow(childWorkflowId);
      const workflows = [mainWorkflow, childWorkflow];

      const result = computeWorkflowPath(workflows, childWorkflowId);

      expect(result).toBe(childWorkflowId);
    });

    it("should handle multiple children under main", () => {
      const child1Id = "subworkflow-1";
      const child2Id = "subworkflow-2";
      const mainWorkflow = createWorkflow(DEFAULT_ENTRY_GRAPH_ID, [
        createNode("node-1", "subworkflow", child1Id),
        createNode("node-2", "subworkflow", child2Id),
      ]);
      const workflows = [
        mainWorkflow,
        createWorkflow(child1Id),
        createWorkflow(child2Id),
      ];

      expect(computeWorkflowPath(workflows, child1Id)).toBe(child1Id);
      expect(computeWorkflowPath(workflows, child2Id)).toBe(child2Id);
    });
  });

  describe("multi-level nesting (deep chains)", () => {
    it("should return dot-separated path for two-level nesting", () => {
      const level1Id = "level-1";
      const level2Id = "level-2";

      const mainWorkflow = createWorkflow(DEFAULT_ENTRY_GRAPH_ID, [
        createNode("main-node", "subworkflow", level1Id),
      ]);
      const level1Workflow = createWorkflow(level1Id, [
        createNode("level1-node", "subworkflow", level2Id),
      ]);
      const level2Workflow = createWorkflow(level2Id);

      const workflows = [mainWorkflow, level1Workflow, level2Workflow];

      const result = computeWorkflowPath(workflows, level2Id);

      expect(result).toBe(`${level1Id}.${level2Id}`);
    });

    it("should return dot-separated path for three-level nesting", () => {
      const level1Id = "level-1";
      const level2Id = "level-2";
      const level3Id = "level-3";

      const mainWorkflow = createWorkflow(DEFAULT_ENTRY_GRAPH_ID, [
        createNode("main-node", "subworkflow", level1Id),
      ]);
      const level1Workflow = createWorkflow(level1Id, [
        createNode("level1-node", "subworkflow", level2Id),
      ]);
      const level2Workflow = createWorkflow(level2Id, [
        createNode("level2-node", "subworkflow", level3Id),
      ]);
      const level3Workflow = createWorkflow(level3Id);

      const workflows = [
        mainWorkflow,
        level1Workflow,
        level2Workflow,
        level3Workflow,
      ];

      const result = computeWorkflowPath(workflows, level3Id);

      expect(result).toBe(`${level1Id}.${level2Id}.${level3Id}`);
    });

    it("should correctly compute path for intermediate workflow in deep chain", () => {
      const level1Id = "level-1";
      const level2Id = "level-2";
      const level3Id = "level-3";

      const mainWorkflow = createWorkflow(DEFAULT_ENTRY_GRAPH_ID, [
        createNode("main-node", "subworkflow", level1Id),
      ]);
      const level1Workflow = createWorkflow(level1Id, [
        createNode("level1-node", "subworkflow", level2Id),
      ]);
      const level2Workflow = createWorkflow(level2Id, [
        createNode("level2-node", "subworkflow", level3Id),
      ]);
      const level3Workflow = createWorkflow(level3Id);

      const workflows = [
        mainWorkflow,
        level1Workflow,
        level2Workflow,
        level3Workflow,
      ];

      expect(computeWorkflowPath(workflows, level1Id)).toBe(level1Id);
      expect(computeWorkflowPath(workflows, level2Id)).toBe(
        `${level1Id}.${level2Id}`,
      );
    });
  });

  describe("edge cases", () => {
    it("should return the workflow id when parent is not found (orphan workflow)", () => {
      const orphanId = "orphan-workflow";
      const workflows = [
        createWorkflow(DEFAULT_ENTRY_GRAPH_ID),
        createWorkflow(orphanId),
      ];

      const result = computeWorkflowPath(workflows, orphanId);

      expect(result).toBe(orphanId);
    });

    it("should handle workflow not in the list", () => {
      const workflows = [createWorkflow(DEFAULT_ENTRY_GRAPH_ID)];

      const result = computeWorkflowPath(workflows, "non-existent");

      expect(result).toBe("non-existent");
    });

    it("should handle empty workflows array", () => {
      const result = computeWorkflowPath([], "some-workflow-id");

      expect(result).toBe("some-workflow-id");
    });

    it("should handle workflows with no nodes", () => {
      const childId = "child-workflow";
      const workflows = [
        createWorkflow(DEFAULT_ENTRY_GRAPH_ID, []),
        createWorkflow(childId, []),
      ];

      const result = computeWorkflowPath(workflows, childId);

      expect(result).toBe(childId);
    });

    it("should ignore non-subworkflow nodes when finding parent", () => {
      const childId = "child-workflow";
      const mainWorkflow = createWorkflow(DEFAULT_ENTRY_GRAPH_ID, [
        createNode("transformer-1", "transformer"),
        createNode("subworkflow-node", "subworkflow", childId),
        createNode("transformer-2", "transformer"),
      ]);
      const workflows = [mainWorkflow, createWorkflow(childId)];

      const result = computeWorkflowPath(workflows, childId);

      expect(result).toBe(childId);
    });
  });

  describe("complex workflow structures", () => {
    it("should handle branching workflow tree", () => {
      //       main
      //      /    \
      //   sub1    sub2
      //    |        |
      //  sub1a    sub2a
      const sub1 = "sub1";
      const sub2 = "sub2";
      const sub1a = "sub1a";
      const sub2a = "sub2a";

      const workflows = [
        createWorkflow(DEFAULT_ENTRY_GRAPH_ID, [
          createNode("n1", "subworkflow", sub1),
          createNode("n2", "subworkflow", sub2),
        ]),
        createWorkflow(sub1, [createNode("n3", "subworkflow", sub1a)]),
        createWorkflow(sub2, [createNode("n4", "subworkflow", sub2a)]),
        createWorkflow(sub1a),
        createWorkflow(sub2a),
      ];

      expect(computeWorkflowPath(workflows, sub1)).toBe(sub1);
      expect(computeWorkflowPath(workflows, sub2)).toBe(sub2);
      expect(computeWorkflowPath(workflows, sub1a)).toBe(`${sub1}.${sub1a}`);
      expect(computeWorkflowPath(workflows, sub2a)).toBe(`${sub2}.${sub2a}`);
    });

    it("should correctly identify the path when same node references subworkflow", () => {
      const childId = "child";
      const workflows = [
        createWorkflow(DEFAULT_ENTRY_GRAPH_ID, [
          createNode("node-ref-to-child", "subworkflow", childId),
        ]),
        createWorkflow(childId, [createNode("internal-node", "transformer")]),
      ];

      expect(computeWorkflowPath(workflows, childId)).toBe(childId);
    });
  });
});
