import * as Y from "yjs";

import type { YNodesMap, YWorkflow } from "@flow/lib/yjs/types";
import type { Node } from "@flow/types";

import { updateNestedSubworkflowPaths } from "./updateNestedSubworkflowPaths";

type NodeConfig = {
  nodeId: string;
  type: string;
  subworkflowId?: string;
};

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

const createYNode = (
  nodeId: string,
  type: string,
  subworkflowId?: string,
): Y.Map<unknown> => {
  const yNode = new Y.Map();
  const yType = new Y.Text();
  yType.insert(0, type);
  yNode.set("type", yType);

  const yData = new Y.Map();
  const yOfficialName = new Y.Text();
  yOfficialName.insert(0, `Node ${nodeId}`);
  yData.set("officialName", yOfficialName);

  if (subworkflowId) {
    const ySubworkflowId = new Y.Text();
    ySubworkflowId.insert(0, subworkflowId);
    yData.set("subworkflowId", ySubworkflowId);
  }

  yNode.set("data", yData);
  return yNode;
};

const createYWorkflow = (
  id: string,
  nodeConfigs: NodeConfig[] = [],
): YWorkflow => {
  const yWorkflow = new Y.Map() as YWorkflow;

  const yId = new Y.Text();
  yId.insert(0, id);
  yWorkflow.set("id", yId);

  const yName = new Y.Text();
  yName.insert(0, `Workflow ${id}`);
  yWorkflow.set("name", yName);

  const yNodes = new Y.Map() as YNodesMap;
  nodeConfigs.forEach(({ nodeId, type, subworkflowId }) => {
    const yNode = createYNode(nodeId, type, subworkflowId);
    yNodes.set(
      nodeId,
      yNode as Y.Map<Y.Text | Y.Map<unknown> | number | boolean>,
    );
  });
  yWorkflow.set("nodes", yNodes);

  const yEdges = new Y.Map();
  yWorkflow.set("edges", yEdges as Y.Map<Y.Map<Y.Text>>);

  return yWorkflow;
};

const getWorkflowPathFromYNode = (
  yNode: Y.Map<unknown>,
): string | undefined => {
  const yData = yNode.get("data") as Y.Map<unknown> | undefined;
  const workflowPath = yData?.get("workflowPath") as Y.Text | undefined;
  return workflowPath?.toJSON();
};

describe("updateNestedSubworkflowPaths", () => {
  let yDoc: Y.Doc;
  let yWorkflows: Y.Map<YWorkflow>;

  beforeEach(() => {
    yDoc = new Y.Doc();
    yWorkflows = yDoc.getMap<YWorkflow>("workflows");
  });

  afterEach(() => {
    yDoc.destroy();
  });

  describe("when no subworkflow nodes exist", () => {
    it("should not modify anything when nodeList is empty", () => {
      const workflow = createYWorkflow("workflow-1", [
        { nodeId: "node-1", type: "transformer" },
      ]);
      yWorkflows.set("workflow-1", workflow);

      updateNestedSubworkflowPaths(yWorkflows, [], "base.path");

      const yNodes = workflow.get("nodes") as YNodesMap;
      const yNode = yNodes.get("node-1");
      expect(yNode).toBeDefined();
      expect(getWorkflowPathFromYNode(yNode as Y.Map<unknown>)).toBeUndefined();
    });

    it("should not modify anything when nodeList has no subworkflow nodes", () => {
      const workflow = createYWorkflow("workflow-1", [
        { nodeId: "node-1", type: "transformer" },
      ]);
      yWorkflows.set("workflow-1", workflow);

      const nodes: Node[] = [createNode("node-2", "transformer")];

      updateNestedSubworkflowPaths(yWorkflows, nodes, "base.path");

      const yNodes = workflow.get("nodes") as YNodesMap;
      const yNode = yNodes.get("node-1");
      expect(yNode).toBeDefined();
      expect(getWorkflowPathFromYNode(yNode as Y.Map<unknown>)).toBeUndefined();
    });
  });

  describe("single-level nesting", () => {
    it("should update workflowPath for nodes in direct child subworkflow", () => {
      const childWorkflowId = "child-workflow";
      const childWorkflow = createYWorkflow(childWorkflowId, [
        { nodeId: "child-node-1", type: "transformer" },
        { nodeId: "child-node-2", type: "transformer" },
      ]);
      yWorkflows.set(childWorkflowId, childWorkflow);

      const parentNodes: Node[] = [
        createNode("parent-node", "subworkflow", childWorkflowId),
      ];

      updateNestedSubworkflowPaths(yWorkflows, parentNodes, "");

      const yNodes = childWorkflow.get("nodes") as YNodesMap;
      const node1 = yNodes.get("child-node-1");
      const node2 = yNodes.get("child-node-2");
      expect(node1).toBeDefined();
      expect(node2).toBeDefined();
      expect(getWorkflowPathFromYNode(node1 as Y.Map<unknown>)).toBe(
        childWorkflowId,
      );
      expect(getWorkflowPathFromYNode(node2 as Y.Map<unknown>)).toBe(
        childWorkflowId,
      );
    });

    it("should prepend basePath to workflowPath when basePath is provided", () => {
      const childWorkflowId = "child-workflow";
      const childWorkflow = createYWorkflow(childWorkflowId, [
        { nodeId: "child-node", type: "transformer" },
      ]);
      yWorkflows.set(childWorkflowId, childWorkflow);

      const parentNodes: Node[] = [
        createNode("parent-node", "subworkflow", childWorkflowId),
      ];

      updateNestedSubworkflowPaths(yWorkflows, parentNodes, "parent.path");

      const yNodes = childWorkflow.get("nodes") as YNodesMap;
      const yNode = yNodes.get("child-node");
      expect(yNode).toBeDefined();
      expect(getWorkflowPathFromYNode(yNode as Y.Map<unknown>)).toBe(
        `parent.path.${childWorkflowId}`,
      );
    });

    it("should handle multiple sibling subworkflows", () => {
      const child1Id = "child-1";
      const child2Id = "child-2";

      const child1 = createYWorkflow(child1Id, [
        { nodeId: "node-in-child-1", type: "transformer" },
      ]);
      const child2 = createYWorkflow(child2Id, [
        { nodeId: "node-in-child-2", type: "transformer" },
      ]);
      yWorkflows.set(child1Id, child1);
      yWorkflows.set(child2Id, child2);

      const parentNodes: Node[] = [
        createNode("ref-to-child-1", "subworkflow", child1Id),
        createNode("ref-to-child-2", "subworkflow", child2Id),
      ];

      updateNestedSubworkflowPaths(yWorkflows, parentNodes, "");

      const child1Nodes = child1.get("nodes") as YNodesMap;
      const child2Nodes = child2.get("nodes") as YNodesMap;
      const nodeInChild1 = child1Nodes.get("node-in-child-1");
      const nodeInChild2 = child2Nodes.get("node-in-child-2");
      expect(nodeInChild1).toBeDefined();
      expect(nodeInChild2).toBeDefined();
      expect(getWorkflowPathFromYNode(nodeInChild1 as Y.Map<unknown>)).toBe(
        child1Id,
      );
      expect(getWorkflowPathFromYNode(nodeInChild2 as Y.Map<unknown>)).toBe(
        child2Id,
      );
    });
  });

  describe("multi-level nesting (recursion)", () => {
    it("should recursively update workflowPath for two-level nesting", () => {
      const level1Id = "level-1";
      const level2Id = "level-2";

      const level1Workflow = createYWorkflow(level1Id, [
        {
          nodeId: "level1-subworkflow-node",
          type: "subworkflow",
          subworkflowId: level2Id,
        },
        { nodeId: "level1-transformer", type: "transformer" },
      ]);
      const level2Workflow = createYWorkflow(level2Id, [
        { nodeId: "level2-node", type: "transformer" },
      ]);

      yWorkflows.set(level1Id, level1Workflow);
      yWorkflows.set(level2Id, level2Workflow);

      const rootNodes: Node[] = [
        createNode("root-node", "subworkflow", level1Id),
      ];

      updateNestedSubworkflowPaths(yWorkflows, rootNodes, "");

      // Level 1 nodes should have path "level-1"
      const level1Nodes = level1Workflow.get("nodes") as YNodesMap;
      const l1SubNode = level1Nodes.get("level1-subworkflow-node");
      const l1Transformer = level1Nodes.get("level1-transformer");
      expect(l1SubNode).toBeDefined();
      expect(l1Transformer).toBeDefined();
      expect(getWorkflowPathFromYNode(l1SubNode as Y.Map<unknown>)).toBe(
        level1Id,
      );
      expect(getWorkflowPathFromYNode(l1Transformer as Y.Map<unknown>)).toBe(
        level1Id,
      );

      // Level 2 nodes should have path "level-1.level-2"
      const level2Nodes = level2Workflow.get("nodes") as YNodesMap;
      const l2Node = level2Nodes.get("level2-node");
      expect(l2Node).toBeDefined();
      expect(getWorkflowPathFromYNode(l2Node as Y.Map<unknown>)).toBe(
        `${level1Id}.${level2Id}`,
      );
    });

    it("should recursively update workflowPath for three-level nesting", () => {
      const level1Id = "level-1";
      const level2Id = "level-2";
      const level3Id = "level-3";

      const level1Workflow = createYWorkflow(level1Id, [
        {
          nodeId: "l1-subworkflow",
          type: "subworkflow",
          subworkflowId: level2Id,
        },
      ]);
      const level2Workflow = createYWorkflow(level2Id, [
        {
          nodeId: "l2-subworkflow",
          type: "subworkflow",
          subworkflowId: level3Id,
        },
      ]);
      const level3Workflow = createYWorkflow(level3Id, [
        { nodeId: "l3-node", type: "transformer" },
      ]);

      yWorkflows.set(level1Id, level1Workflow);
      yWorkflows.set(level2Id, level2Workflow);
      yWorkflows.set(level3Id, level3Workflow);

      const rootNodes: Node[] = [
        createNode("root-node", "subworkflow", level1Id),
      ];

      updateNestedSubworkflowPaths(yWorkflows, rootNodes, "");

      const l1Nodes = level1Workflow.get("nodes") as YNodesMap;
      const l2Nodes = level2Workflow.get("nodes") as YNodesMap;
      const l3Nodes = level3Workflow.get("nodes") as YNodesMap;

      const l1Sub = l1Nodes.get("l1-subworkflow");
      const l2Sub = l2Nodes.get("l2-subworkflow");
      const l3Node = l3Nodes.get("l3-node");

      expect(l1Sub).toBeDefined();
      expect(l2Sub).toBeDefined();
      expect(l3Node).toBeDefined();

      expect(getWorkflowPathFromYNode(l1Sub as Y.Map<unknown>)).toBe(level1Id);
      expect(getWorkflowPathFromYNode(l2Sub as Y.Map<unknown>)).toBe(
        `${level1Id}.${level2Id}`,
      );
      expect(getWorkflowPathFromYNode(l3Node as Y.Map<unknown>)).toBe(
        `${level1Id}.${level2Id}.${level3Id}`,
      );
    });

    it("should handle branching workflow tree", () => {
      //       root
      //      /    \
      //   sub1    sub2
      //    |        |
      //  sub1a    sub2a
      const sub1 = "sub1";
      const sub2 = "sub2";
      const sub1a = "sub1a";
      const sub2a = "sub2a";

      const sub1Workflow = createYWorkflow(sub1, [
        { nodeId: "sub1-child-ref", type: "subworkflow", subworkflowId: sub1a },
      ]);
      const sub2Workflow = createYWorkflow(sub2, [
        { nodeId: "sub2-child-ref", type: "subworkflow", subworkflowId: sub2a },
      ]);
      const sub1aWorkflow = createYWorkflow(sub1a, [
        { nodeId: "leaf1", type: "transformer" },
      ]);
      const sub2aWorkflow = createYWorkflow(sub2a, [
        { nodeId: "leaf2", type: "transformer" },
      ]);

      yWorkflows.set(sub1, sub1Workflow);
      yWorkflows.set(sub2, sub2Workflow);
      yWorkflows.set(sub1a, sub1aWorkflow);
      yWorkflows.set(sub2a, sub2aWorkflow);

      const rootNodes: Node[] = [
        createNode("ref-sub1", "subworkflow", sub1),
        createNode("ref-sub2", "subworkflow", sub2),
      ];

      updateNestedSubworkflowPaths(yWorkflows, rootNodes, "");

      // Check sub1 branch
      const sub1Nodes = sub1Workflow.get("nodes") as YNodesMap;
      const sub1aNodes = sub1aWorkflow.get("nodes") as YNodesMap;
      const sub1ChildRef = sub1Nodes.get("sub1-child-ref");
      const leaf1 = sub1aNodes.get("leaf1");

      expect(sub1ChildRef).toBeDefined();
      expect(leaf1).toBeDefined();
      expect(getWorkflowPathFromYNode(sub1ChildRef as Y.Map<unknown>)).toBe(
        sub1,
      );
      expect(getWorkflowPathFromYNode(leaf1 as Y.Map<unknown>)).toBe(
        `${sub1}.${sub1a}`,
      );

      // Check sub2 branch
      const sub2Nodes = sub2Workflow.get("nodes") as YNodesMap;
      const sub2aNodes = sub2aWorkflow.get("nodes") as YNodesMap;
      const sub2ChildRef = sub2Nodes.get("sub2-child-ref");
      const leaf2 = sub2aNodes.get("leaf2");

      expect(sub2ChildRef).toBeDefined();
      expect(leaf2).toBeDefined();
      expect(getWorkflowPathFromYNode(sub2ChildRef as Y.Map<unknown>)).toBe(
        sub2,
      );
      expect(getWorkflowPathFromYNode(leaf2 as Y.Map<unknown>)).toBe(
        `${sub2}.${sub2a}`,
      );
    });
  });

  describe("edge cases", () => {
    it("should handle missing subworkflowId gracefully", () => {
      const childWorkflowId = "child-workflow";
      const childWorkflow = createYWorkflow(childWorkflowId, [
        { nodeId: "child-node", type: "transformer" },
      ]);
      yWorkflows.set(childWorkflowId, childWorkflow);

      const parentNodes: Node[] = [
        {
          id: "node-without-subworkflowId",
          type: "subworkflow",
          position: { x: 0, y: 0 },
          data: {
            officialName: "Missing subworkflowId",
            // subworkflowId is intentionally missing
          },
        },
      ];

      // Should not throw and should not modify anything
      expect(() =>
        updateNestedSubworkflowPaths(yWorkflows, parentNodes, ""),
      ).not.toThrow();

      const yNodes = childWorkflow.get("nodes") as YNodesMap;
      const yNode = yNodes.get("child-node");
      expect(yNode).toBeDefined();
      expect(getWorkflowPathFromYNode(yNode as Y.Map<unknown>)).toBeUndefined();
    });

    it("should handle non-existent subworkflow reference gracefully", () => {
      const parentNodes: Node[] = [
        createNode("parent-node", "subworkflow", "non-existent-workflow"),
      ];

      // Should not throw
      expect(() =>
        updateNestedSubworkflowPaths(yWorkflows, parentNodes, ""),
      ).not.toThrow();
    });

    it("should handle workflow with empty nodes map", () => {
      const childWorkflowId = "empty-child";
      const childWorkflow = createYWorkflow(childWorkflowId, []);
      yWorkflows.set(childWorkflowId, childWorkflow);

      const parentNodes: Node[] = [
        createNode("parent-node", "subworkflow", childWorkflowId),
      ];

      expect(() =>
        updateNestedSubworkflowPaths(yWorkflows, parentNodes, ""),
      ).not.toThrow();
    });

    it("should handle workflow with missing nodes map", () => {
      const childWorkflowId = "malformed-child";
      const yWorkflow = new Y.Map() as YWorkflow;
      const yId = new Y.Text();
      yId.insert(0, childWorkflowId);
      yWorkflow.set("id", yId);
      // Intentionally not setting "nodes"
      yWorkflows.set(childWorkflowId, yWorkflow);

      const parentNodes: Node[] = [
        createNode("parent-node", "subworkflow", childWorkflowId),
      ];

      expect(() =>
        updateNestedSubworkflowPaths(yWorkflows, parentNodes, ""),
      ).not.toThrow();
    });

    it("should handle nodes with missing data map", () => {
      const childWorkflowId = "child-with-bad-node";
      const yWorkflow = new Y.Map() as YWorkflow;
      const yId = new Y.Text();
      yId.insert(0, childWorkflowId);
      yWorkflow.set("id", yId);

      const yNodes = new Y.Map() as YNodesMap;
      const badYNode = new Y.Map();
      badYNode.set("type", new Y.Text());
      // Intentionally not setting "data"
      yNodes.set(
        "bad-node",
        badYNode as Y.Map<Y.Text | Y.Map<unknown> | number | boolean>,
      );
      yWorkflow.set("nodes", yNodes);

      yWorkflows.set(childWorkflowId, yWorkflow);

      const parentNodes: Node[] = [
        createNode("parent-node", "subworkflow", childWorkflowId),
      ];

      expect(() =>
        updateNestedSubworkflowPaths(yWorkflows, parentNodes, ""),
      ).not.toThrow();
    });

    it("should update workflowPath on nodes that already have it set", () => {
      const childWorkflowId = "child-workflow";
      const childWorkflow = createYWorkflow(childWorkflowId, [
        { nodeId: "child-node", type: "transformer" },
      ]);

      // First add to yWorkflows so Yjs types are properly attached to the document
      yWorkflows.set(childWorkflowId, childWorkflow);

      // Now pre-set an existing workflowPath
      const yNodes = childWorkflow.get("nodes") as YNodesMap;
      const yNode = yNodes.get("child-node");
      expect(yNode).toBeDefined();
      const yData = (yNode as Y.Map<unknown>).get("data") as Y.Map<unknown>;
      const existingPath = new Y.Text();
      existingPath.insert(0, "old.path");
      yData.set("workflowPath", existingPath);

      const parentNodes: Node[] = [
        createNode("parent-node", "subworkflow", childWorkflowId),
      ];

      updateNestedSubworkflowPaths(yWorkflows, parentNodes, "new.base");

      // Should overwrite with new path
      const updatedNode = yNodes.get("child-node");
      expect(updatedNode).toBeDefined();
      expect(getWorkflowPathFromYNode(updatedNode as Y.Map<unknown>)).toBe(
        `new.base.${childWorkflowId}`,
      );
    });
  });

  describe("Yjs mutation behavior", () => {
    it("should directly mutate Yjs maps (not create copies)", () => {
      const childWorkflowId = "child-workflow";
      const childWorkflow = createYWorkflow(childWorkflowId, [
        { nodeId: "child-node", type: "transformer" },
      ]);
      yWorkflows.set(childWorkflowId, childWorkflow);

      const yNodesBefore = childWorkflow.get("nodes") as YNodesMap;
      const yNodeBefore = yNodesBefore.get("child-node");

      const parentNodes: Node[] = [
        createNode("parent-node", "subworkflow", childWorkflowId),
      ];

      updateNestedSubworkflowPaths(yWorkflows, parentNodes, "");

      const yNodesAfter = childWorkflow.get("nodes") as YNodesMap;
      const yNodeAfter = yNodesAfter.get("child-node");

      // Should be the same Y.Map instance (mutated, not replaced)
      expect(yNodeAfter).toBe(yNodeBefore);
      expect(yNodeAfter).toBeDefined();
      expect(getWorkflowPathFromYNode(yNodeAfter as Y.Map<unknown>)).toBe(
        childWorkflowId,
      );
    });

    it("should use Y.Text for workflowPath values", () => {
      const childWorkflowId = "child-workflow";
      const childWorkflow = createYWorkflow(childWorkflowId, [
        { nodeId: "child-node", type: "transformer" },
      ]);
      yWorkflows.set(childWorkflowId, childWorkflow);

      const parentNodes: Node[] = [
        createNode("parent-node", "subworkflow", childWorkflowId),
      ];

      updateNestedSubworkflowPaths(yWorkflows, parentNodes, "");

      const yNodes = childWorkflow.get("nodes") as YNodesMap;
      const yNode = yNodes.get("child-node");
      expect(yNode).toBeDefined();
      const yData = (yNode as Y.Map<unknown>).get("data") as Y.Map<unknown>;
      const workflowPath = yData.get("workflowPath");

      expect(workflowPath).toBeInstanceOf(Y.Text);
    });
  });
});
