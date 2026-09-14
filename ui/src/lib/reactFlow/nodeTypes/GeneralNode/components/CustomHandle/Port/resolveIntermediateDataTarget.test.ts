import type { NodeData } from "@flow/types";

import {
  intermediateDataArtifactUrl,
  resolveIntermediateDataTarget,
} from "./resolveIntermediateDataTarget";

const nodeData = (overrides: Partial<NodeData> = {}): NodeData => ({
  officialName: "Node",
  ...overrides,
});

describe("resolveIntermediateDataTarget", () => {
  describe("ordinary nodes", () => {
    it("keeps the node's own coordinates in the main workflow", () => {
      expect(
        resolveIntermediateDataTarget({
          nodeId: "node-1",
          nodeData: nodeData(),
          portName: "default",
        }),
      ).toEqual({ nodeId: "node-1", portName: "default", workflowPath: "" });
    });

    it("keeps the node's own coordinates inside a subworkflow", () => {
      expect(
        resolveIntermediateDataTarget({
          nodeId: "node-1",
          nodeData: nodeData({ workflowPath: "sub-a" }),
          portName: "default",
        }),
      ).toEqual({
        nodeId: "node-1",
        portName: "default",
        workflowPath: "sub-a",
      });
    });
  });

  describe("subworkflow nodes", () => {
    it("redirects a pseudo port to the child workflow's output router", () => {
      expect(
        resolveIntermediateDataTarget({
          nodeId: "sub-a",
          nodeData: nodeData({
            subworkflowId: "sub-a",
            workflowPath: "",
            pseudoOutputs: [{ nodeId: "router-out", portName: "default" }],
          }),
          portName: "default",
        }),
      ).toEqual({
        nodeId: "router-out",
        portName: "default",
        workflowPath: "sub-a",
      });
    });

    it("nests the child path under the parent's", () => {
      expect(
        resolveIntermediateDataTarget({
          nodeId: "sub-b",
          nodeData: nodeData({
            subworkflowId: "sub-b",
            workflowPath: "sub-a",
            pseudoOutputs: [{ nodeId: "router-out", portName: "default" }],
          }),
          portName: "default",
        }),
      ).toEqual({
        nodeId: "router-out",
        portName: "default",
        workflowPath: "sub-a.sub-b",
      });
    });

    it("picks the router matching the handle when there are several", () => {
      const data = nodeData({
        subworkflowId: "sub-a",
        pseudoOutputs: [
          { nodeId: "router-1", portName: "left" },
          { nodeId: "router-2", portName: "right" },
        ],
      });

      expect(
        resolveIntermediateDataTarget({
          nodeId: "sub-a",
          nodeData: data,
          portName: "right",
        }),
      ).toMatchObject({ nodeId: "router-2", portName: "right" });
    });

    it("uses the subworkflow id, not the node id, to build the child path", () => {
      // Imported engine workflows give the subworkflow node an id of its own,
      // distinct from the graph it points at.
      expect(
        resolveIntermediateDataTarget({
          nodeId: "engine-node-7",
          nodeData: nodeData({
            subworkflowId: "graph-42",
            pseudoOutputs: [{ nodeId: "router-out", portName: "default" }],
          }),
          portName: "default",
        }),
      ).toEqual({
        nodeId: "router-out",
        portName: "default",
        workflowPath: "graph-42",
      });
    });

    it("falls back to the node's own coordinates for an unmatched handle", () => {
      expect(
        resolveIntermediateDataTarget({
          nodeId: "sub-a",
          nodeData: nodeData({
            subworkflowId: "sub-a",
            pseudoOutputs: [{ nodeId: "router-out", portName: "default" }],
          }),
          portName: "unknown",
        }),
      ).toEqual({ nodeId: "sub-a", portName: "unknown", workflowPath: "" });
    });
  });
});

describe("intermediateDataArtifactUrl", () => {
  it("omits the path segment in the main workflow", () => {
    expect(
      intermediateDataArtifactUrl("https://api", "job-1", {
        nodeId: "node-1",
        portName: "default",
        workflowPath: "",
      }),
    ).toBe(
      "https://api/artifacts/job-1/feature-store/node-1.default.jsonl.zst",
    );
  });

  it("prefixes the workflow path when nested", () => {
    expect(
      intermediateDataArtifactUrl("https://api", "job-1", {
        nodeId: "router-out",
        portName: "default",
        workflowPath: "sub-a.sub-b",
      }),
    ).toBe(
      "https://api/artifacts/job-1/feature-store/sub-a.sub-b.router-out.default.jsonl.zst",
    );
  });
});
