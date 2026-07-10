import { describe, expect, it } from "vitest";
import * as Y from "yjs";

import type { Edge, Node } from "@flow/types";

import { yWorkflowConstructor } from "../conversions";
import { rebuildWorkflow } from "../conversions/rebuildWorkflow";
import type { YWorkflow } from "../types";

import { hasLegacyPorts, migrateLegacyPorts } from "./legacyPortsMigration";

const node = (id: string, data: Node["data"]): Node => ({
  id,
  type: "transformer",
  position: { x: 0, y: 0 },
  data,
});

const buildDoc = (nodes: Node[], edges: Edge[] = []) => {
  const yDoc = new Y.Doc();
  const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
  yWorkflows.set("main", yWorkflowConstructor("main", "Main", nodes, edges));
  return yWorkflows;
};

describe("hasLegacyPorts", () => {
  it("returns false for a project already on the features port", () => {
    const yWorkflows = buildDoc(
      [
        node("n1", {
          officialName: "FeatureCreator",
          inputs: ["features"],
          outputs: ["features"],
        }),
      ],
      [
        {
          id: "e1",
          source: "n1",
          target: "n2",
          sourceHandle: "features",
          targetHandle: "features",
        },
      ],
    );
    expect(hasLegacyPorts(yWorkflows)).toBe(false);
  });

  it("detects legacy edge handles", () => {
    const yWorkflows = buildDoc(
      [],
      [{ id: "e1", source: "a", target: "b", sourceHandle: "default" }],
    );
    expect(hasLegacyPorts(yWorkflows)).toBe(true);
  });

  it("detects legacy node input/output ports", () => {
    const yWorkflows = buildDoc([
      node("n1", { officialName: "X", outputs: ["default"] }),
    ]);
    expect(hasLegacyPorts(yWorkflows)).toBe(true);
  });

  it("detects legacy routingPort params", () => {
    const yWorkflows = buildDoc([
      node("n1", {
        officialName: "InputRouter",
        params: { routingPort: "default" },
      }),
    ]);
    expect(hasLegacyPorts(yWorkflows)).toBe(true);
  });

  it("detects legacy pseudo port names", () => {
    const yWorkflows = buildDoc([
      node("n1", {
        officialName: "Subworkflow",
        pseudoInputs: [{ nodeId: "r1", portName: "default" }],
      }),
    ]);
    expect(hasLegacyPorts(yWorkflows)).toBe(true);
  });

  it("ignores composed pseudo port names ending in -default", () => {
    const yWorkflows = buildDoc(
      [
        node("n1", {
          officialName: "InputRouter",
          params: { routingPort: "MyNode-default" },
        }),
      ],
      [
        {
          id: "e1",
          source: "a",
          target: "b",
          targetHandle: "MyNode-default",
        },
      ],
    );
    expect(hasLegacyPorts(yWorkflows)).toBe(false);
  });

  it("ignores user-named condition ports and their edges (e.g. FeatureFilter output named 'default')", () => {
    const yWorkflows = buildDoc(
      [
        node("filter", {
          officialName: "FeatureFilter",
          params: {
            conditions: [
              { expr: "true", outputPort: "default" },
              { expr: "false", outputPort: "other" },
            ],
          },
        }),
        node("merger", {
          officialName: "FeatureMerger",
          params: {
            conditions: [{ inputPort: "default" }],
          },
        }),
      ],
      [
        {
          id: "e1",
          source: "filter",
          target: "merger",
          sourceHandle: "default",
          targetHandle: "default",
        },
      ],
    );
    expect(hasLegacyPorts(yWorkflows)).toBe(false);
  });
});

describe("migrateLegacyPorts", () => {
  it("rewrites all legacy port references and leaves the rest untouched", () => {
    const yWorkflows = buildDoc(
      [
        node("n1", {
          officialName: "AttributeManager",
          inputs: ["default"],
          outputs: ["default", "rejected"],
        }),
        node("router", {
          officialName: "InputRouter",
          params: { routingPort: "default", other: "default" },
        }),
        node("sub", {
          officialName: "Subworkflow",
          pseudoInputs: [{ nodeId: "router", portName: "default" }],
          pseudoOutputs: [{ nodeId: "out", portName: "MyNode-default" }],
        }),
      ],
      [
        {
          id: "e1",
          source: "n1",
          target: "sub",
          sourceHandle: "default",
          targetHandle: "default",
        },
        {
          id: "e2",
          source: "n1",
          target: "x",
          sourceHandle: "rejected",
        },
      ],
    );

    const changed = migrateLegacyPorts(yWorkflows);
    expect(changed).toBe(6);
    expect(hasLegacyPorts(yWorkflows)).toBe(false);

    const workflow = rebuildWorkflow(yWorkflows.get("main") as YWorkflow);
    const nodes = workflow.nodes as Node[];
    const edges = workflow.edges as Edge[];

    const n1 = nodes.find((n) => n.id === "n1");
    expect(n1?.data.inputs).toEqual(["features"]);
    expect(n1?.data.outputs).toEqual(["features", "rejected"]);

    const router = nodes.find((n) => n.id === "router");
    expect(router?.data.params?.routingPort).toBe("features");
    // Only routingPort is a port reference — other params keep their values
    expect(router?.data.params?.other).toBe("default");

    const sub = nodes.find((n) => n.id === "sub");
    expect(sub?.data.pseudoInputs?.[0].portName).toBe("features");
    expect(sub?.data.pseudoOutputs?.[0].portName).toBe("MyNode-default");

    const e1 = edges.find((e) => e.id === "e1");
    expect(e1?.sourceHandle).toBe("features");
    expect(e1?.targetHandle).toBe("features");

    const e2 = edges.find((e) => e.id === "e2");
    expect(e2?.sourceHandle).toBe("rejected");
  });

  it("preserves a custom condition port named 'default' while migrating legacy ports on other nodes", () => {
    const yWorkflows = buildDoc(
      [
        node("filter", {
          officialName: "FeatureFilter",
          inputs: ["default"],
          params: {
            conditions: [{ expr: "true", outputPort: "default" }],
          },
        }),
        node("writer", {
          officialName: "SomeWriter",
          inputs: ["default"],
        }),
      ],
      [
        // Out of the filter's user-named "default" port — must be preserved
        {
          id: "e1",
          source: "filter",
          target: "writer",
          sourceHandle: "default",
          targetHandle: "default",
        },
        // Into the filter's legacy action-definition input — must be migrated
        {
          id: "e2",
          source: "reader",
          target: "filter",
          sourceHandle: "default",
          targetHandle: "default",
        },
      ],
    );

    // 2 node input lists + e1 targetHandle + e2 sourceHandle + e2 targetHandle
    expect(migrateLegacyPorts(yWorkflows)).toBe(5);

    const workflow = rebuildWorkflow(yWorkflows.get("main") as YWorkflow);
    const edges = workflow.edges as Edge[];

    const e1 = edges.find((e) => e.id === "e1");
    expect(e1?.sourceHandle).toBe("default");
    expect(e1?.targetHandle).toBe("features");

    const e2 = edges.find((e) => e.id === "e2");
    expect(e2?.sourceHandle).toBe("features");
    expect(e2?.targetHandle).toBe("features");

    const filter = (workflow.nodes as Node[]).find((n) => n.id === "filter");
    expect(filter?.data.params?.conditions[0].outputPort).toBe("default");
    expect(filter?.data.inputs).toEqual(["features"]);
  });

  it("is idempotent", () => {
    const yWorkflows = buildDoc(
      [],
      [{ id: "e1", source: "a", target: "b", sourceHandle: "default" }],
    );
    expect(migrateLegacyPorts(yWorkflows)).toBe(1);
    expect(migrateLegacyPorts(yWorkflows)).toBe(0);
  });
});
