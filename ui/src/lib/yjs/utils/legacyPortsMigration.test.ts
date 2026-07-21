import { describe, expect, it } from "vitest";
import * as Y from "yjs";

import type { Edge, Node, NodeType } from "@flow/types";

import { yWorkflowConstructor } from "../conversions";
import { rebuildWorkflow } from "../conversions/rebuildWorkflow";
import type { YWorkflow } from "../types";

import { hasLegacyPorts, migrateLegacyPorts } from "./legacyPortsMigration";

const node = (
  id: string,
  data: Node["data"],
  type: NodeType = "transformer",
): Node => ({
  id,
  type,
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

  it("ignores a routingPort named 'default' — user-named router ports stay legal", () => {
    const yWorkflows = buildDoc([
      node("n1", {
        officialName: "InputRouter",
        outputs: ["features"],
        params: { routingPort: "default" },
      }),
    ]);
    expect(hasLegacyPorts(yWorkflows)).toBe(false);
  });

  it("ignores Input/OutputRouter nodes entirely — existing routers keep their 'default' handles", () => {
    const yWorkflows = buildDoc(
      [
        node("in", {
          officialName: "InputRouter",
          outputs: ["default"],
          params: { routingPort: "features" },
        }),
        node("out", {
          officialName: "OutputRouter",
          inputs: ["default"],
          params: { routingPort: "features" },
        }),
        node("n1", {
          officialName: "AttributeManager",
          inputs: ["features"],
          outputs: ["features"],
        }),
      ],
      [
        {
          id: "e1",
          source: "in",
          target: "n1",
          sourceHandle: "default",
          targetHandle: "features",
        },
        {
          id: "e2",
          source: "n1",
          target: "out",
          sourceHandle: "features",
          targetHandle: "default",
        },
      ],
    );
    expect(hasLegacyPorts(yWorkflows)).toBe(false);
  });

  it("ignores pseudo port names and subworkflow edge handles named 'default'", () => {
    const yWorkflows = buildDoc(
      [
        node(
          "sub",
          {
            officialName: "Subworkflow",
            pseudoInputs: [{ nodeId: "r1", portName: "default" }],
            pseudoOutputs: [{ nodeId: "r2", portName: "default" }],
          },
          "subworkflow",
        ),
        node("n1", { officialName: "X", outputs: ["features"] }),
      ],
      [
        {
          id: "e1",
          source: "n1",
          target: "sub",
          sourceHandle: "features",
          targetHandle: "default",
        },
        {
          id: "e2",
          source: "sub",
          target: "n1",
          sourceHandle: "default",
          targetHandle: "features",
        },
      ],
    );
    expect(hasLegacyPorts(yWorkflows)).toBe(false);
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
  it("rewrites action-definition ports and leaves user-named ports untouched", () => {
    const yWorkflows = buildDoc(
      [
        node("n1", {
          officialName: "AttributeManager",
          inputs: ["default"],
          outputs: ["default", "rejected"],
        }),
        // Existing router: kept exactly as-is — handles and routingPort
        node("router", {
          officialName: "InputRouter",
          outputs: ["default"],
          params: { routingPort: "default" },
        }),
        node(
          "sub",
          {
            officialName: "Subworkflow",
            pseudoInputs: [{ nodeId: "router", portName: "default" }],
            pseudoOutputs: [{ nodeId: "out", portName: "MyNode-default" }],
          },
          "subworkflow",
        ),
      ],
      [
        // n1's action port (migrate) → sub's pseudo port (preserve)
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
        // router side preserved, n1 side migrated
        {
          id: "e3",
          source: "router",
          target: "n1",
          sourceHandle: "default",
          targetHandle: "default",
        },
      ],
    );

    // n1 inputs + n1 outputs + e1 sourceHandle + e3 targetHandle
    const changed = migrateLegacyPorts(yWorkflows);
    expect(changed).toBe(4);
    expect(hasLegacyPorts(yWorkflows)).toBe(false);

    const workflow = rebuildWorkflow(yWorkflows.get("main") as YWorkflow);
    const nodes = workflow.nodes as Node[];
    const edges = workflow.edges as Edge[];

    const n1 = nodes.find((n) => n.id === "n1");
    expect(n1?.data.inputs).toEqual(["features"]);
    expect(n1?.data.outputs).toEqual(["features", "rejected"]);

    const router = nodes.find((n) => n.id === "router");
    expect(router?.data.outputs).toEqual(["default"]);
    expect(router?.data.params?.routingPort).toBe("default");

    const sub = nodes.find((n) => n.id === "sub");
    expect(sub?.data.pseudoInputs?.[0].portName).toBe("default");
    expect(sub?.data.pseudoOutputs?.[0].portName).toBe("MyNode-default");

    const e1 = edges.find((e) => e.id === "e1");
    expect(e1?.sourceHandle).toBe("features");
    expect(e1?.targetHandle).toBe("default");

    const e2 = edges.find((e) => e.id === "e2");
    expect(e2?.sourceHandle).toBe("rejected");

    const e3 = edges.find((e) => e.id === "e3");
    expect(e3?.sourceHandle).toBe("default");
    expect(e3?.targetHandle).toBe("features");
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
