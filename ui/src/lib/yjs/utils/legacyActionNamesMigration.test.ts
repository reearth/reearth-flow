import { describe, expect, it } from "vitest";
import * as Y from "yjs";

import type { Node, NodeType } from "@flow/types";

import { yWorkflowConstructor } from "../conversions";
import { rebuildWorkflow } from "../conversions/rebuildWorkflow";
import type { YWorkflow } from "../types";

import {
  hasLegacyActionNames,
  migrateLegacyActionNames,
} from "./legacyActionNamesMigration";

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

const buildDoc = (nodes: Node[]) => {
  const yDoc = new Y.Doc();
  const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
  yWorkflows.set("main", yWorkflowConstructor("main", "Main", nodes));
  return yWorkflows;
};

describe("hasLegacyActionNames", () => {
  it("returns false for a project already on renamed actions", () => {
    const yWorkflows = buildDoc([
      node("n1", { officialName: "Feature Creator" }, "reader"),
      node("n2", { officialName: "Attribute Manager" }),
      node("n3", { officialName: "Cesium 3D Tiles Writer" }, "writer"),
    ]);
    expect(hasLegacyActionNames(yWorkflows)).toBe(false);
  });

  it("detects pre-rename action names", () => {
    const yWorkflows = buildDoc([
      node("n1", { officialName: "Cesium3DTilesWriter" }, "writer"),
    ]);
    expect(hasLegacyActionNames(yWorkflows)).toBe(true);
  });

  it("ignores non-action nodes — their officialName is user space", () => {
    const yWorkflows = buildDoc([
      node("sub", { officialName: "FeatureFilter" }, "subworkflow"),
      node("note", { officialName: "FeatureFilter" }, "note"),
      node("batch", { officialName: "FeatureFilter" }, "batch"),
    ]);
    expect(hasLegacyActionNames(yWorkflows)).toBe(false);
  });
});

describe("migrateLegacyActionNames", () => {
  it("rewrites legacy names on action nodes and leaves everything else untouched", () => {
    const yWorkflows = buildDoc([
      node("n1", { officialName: "CityGmlReader" }, "reader"),
      node("n2", {
        officialName: "FeatureLodFilter",
        customizations: { customName: "My FeatureLodFilter" },
      }),
      node("router", { officialName: "InputRouter" }),
      node("n3", { officialName: "Bufferer" }),
      node("sub", { officialName: "FeatureFilter" }, "subworkflow"),
    ]);

    const changed = migrateLegacyActionNames(yWorkflows);
    expect(changed).toBe(3);
    expect(hasLegacyActionNames(yWorkflows)).toBe(false);

    const workflow = rebuildWorkflow(yWorkflows.get("main") as YWorkflow);
    const nodes = workflow.nodes as Node[];
    const byId = (id: string) => nodes.find((n) => n.id === id);

    expect(byId("n1")?.data.officialName).toBe("CityGML Reader");
    expect(byId("n2")?.data.officialName).toBe("Feature LOD Filter");
    // User-set display names are not action names
    expect(byId("n2")?.data.customizations?.customName).toBe(
      "My FeatureLodFilter",
    );
    expect(byId("router")?.data.officialName).toBe("Input Router");
    // Unrenamed action, user-named subworkflow: unchanged
    expect(byId("n3")?.data.officialName).toBe("Bufferer");
    expect(byId("sub")?.data.officialName).toBe("FeatureFilter");
  });

  it("is idempotent", () => {
    const yWorkflows = buildDoc([
      node("n1", { officialName: "GeoJsonWriter" }, "writer"),
    ]);
    expect(migrateLegacyActionNames(yWorkflows)).toBe(1);
    expect(migrateLegacyActionNames(yWorkflows)).toBe(0);
  });
});
