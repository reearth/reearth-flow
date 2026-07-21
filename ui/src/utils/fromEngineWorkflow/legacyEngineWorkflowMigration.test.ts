import { describe, expect, it } from "vitest";

import type { EngineReadyWorkflow } from "@flow/types";

import {
  isLegacyEngineWorkflow,
  migrateLegacyEngineWorkflow,
} from "./legacyEngineWorkflowMigration";

const workflow = (
  graphs: EngineReadyWorkflow["graphs"],
): EngineReadyWorkflow => ({
  id: "w1",
  name: "Test",
  entryGraphId: "g1",
  graphs,
});

describe("isLegacyEngineWorkflow", () => {
  it("returns false for a file using renamed actions", () => {
    const engineWorkflow = workflow([
      {
        id: "g1",
        name: "Main",
        nodes: [
          { id: "n1", name: "Reader", type: "action", action: "CSV Reader" },
        ],
        edges: [],
      },
    ]);
    expect(isLegacyEngineWorkflow(engineWorkflow)).toBe(false);
  });

  it("detects pre-rename action names in any graph", () => {
    const engineWorkflow = workflow([
      { id: "g1", name: "Main", nodes: [], edges: [] },
      {
        id: "g2",
        name: "Sub",
        nodes: [
          { id: "n1", name: "Writer", type: "action", action: "CsvWriter" },
        ],
        edges: [],
      },
    ]);
    expect(isLegacyEngineWorkflow(engineWorkflow)).toBe(true);
  });
});

describe("migrateLegacyEngineWorkflow", () => {
  it("returns a non-legacy file untouched, including 'default' user port names", () => {
    const engineWorkflow = workflow([
      {
        id: "g1",
        name: "Main",
        nodes: [
          {
            id: "n1",
            name: "Filter",
            type: "action",
            action: "Feature Filter",
            with: { conditions: [{ expr: "true", outputPort: "default" }] },
          },
        ],
        edges: [
          { id: "e1", from: "n1", to: "n2", fromPort: "default", toPort: "x" },
        ],
      },
    ]);
    expect(migrateLegacyEngineWorkflow(engineWorkflow)).toBe(engineWorkflow);
  });

  it("rewrites action names and every 'default' port field in a legacy file", () => {
    const engineWorkflow = workflow([
      {
        id: "g1",
        name: "Main",
        nodes: [
          {
            id: "reader",
            name: "My Reader",
            type: "action",
            action: "CityGmlReader",
            with: { citygmlPath: "a.gml" },
          },
          {
            id: "filter",
            name: "Filter",
            type: "action",
            action: "FeatureFilter",
            with: {
              conditions: [
                { expr: "true", outputPort: "default" },
                { expr: "false", outputPort: "other" },
              ],
            },
          },
          { id: "sub", name: "Sub", type: "subGraph", subGraphId: "g2" },
        ],
        edges: [
          {
            id: "e1",
            from: "reader",
            to: "filter",
            fromPort: "default",
            toPort: "default",
          },
          {
            id: "e2",
            from: "filter",
            to: "sub",
            fromPort: "other",
            toPort: "default",
          },
        ],
      },
      {
        id: "g2",
        name: "Sub",
        nodes: [
          {
            id: "router",
            name: "Entry",
            type: "action",
            action: "InputRouter",
            with: { routingPort: "default" },
          },
        ],
        edges: [],
      },
    ]);

    const migrated = migrateLegacyEngineWorkflow(engineWorkflow);

    const [g1, g2] = migrated.graphs;
    expect(g1.nodes[0].action).toBe("CityGML Reader");
    expect(g1.nodes[0].with).toEqual({ citygmlPath: "a.gml" });
    expect(g1.nodes[1].action).toBe("Feature Filter");
    // Condition ports rename together with the edges referencing them
    expect(g1.nodes[1].with.conditions).toEqual([
      { expr: "true", outputPort: "features" },
      { expr: "false", outputPort: "other" },
    ]);
    // subGraph node and user labels untouched
    expect(g1.nodes[2]).toEqual(engineWorkflow.graphs[0].nodes[2]);
    expect(g1.nodes[0].name).toBe("My Reader");

    expect(g1.edges[0].fromPort).toBe("features");
    expect(g1.edges[0].toPort).toBe("features");
    expect(g1.edges[1].fromPort).toBe("other");
    // Router routingPort renames together with the parent subGraph edge port
    expect(g1.edges[1].toPort).toBe("features");
    expect(g2.nodes[0].action).toBe("Input Router");
    expect(g2.nodes[0].with.routingPort).toBe("features");

    // Input file is not mutated
    expect(engineWorkflow.graphs[0].nodes[0].action).toBe("CityGmlReader");
    expect(engineWorkflow.graphs[0].edges[0].fromPort).toBe("default");
  });

  it("is idempotent", () => {
    const engineWorkflow = workflow([
      {
        id: "g1",
        name: "Main",
        nodes: [
          { id: "n1", name: "Writer", type: "action", action: "XmlWriter" },
        ],
        edges: [
          {
            id: "e1",
            from: "n0",
            to: "n1",
            fromPort: "default",
            toPort: "default",
          },
        ],
      },
    ]);
    const once = migrateLegacyEngineWorkflow(engineWorkflow);
    const twice = migrateLegacyEngineWorkflow(once);
    expect(twice).toBe(once);
    expect(once.graphs[0].nodes[0].action).toBe("XML Writer");
  });
});
