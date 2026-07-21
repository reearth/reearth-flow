import * as Y from "yjs";

import { DEFAULT_EDGE_PORT } from "@flow/global-constants";

import type { YWorkflow } from "../types";

// Port name used before the engine renamed its default port to "features" (engine PR #2236).
const LEGACY_PORT = "default";

// User-defined condition ports (e.g. FeatureFilter outputPort) may legitimately
// be named "default" — never rewrite these or their edges.
const getConditionPorts = (
  params: unknown,
  key: "inputPort" | "outputPort",
): string[] => {
  const conditions = (params as Record<string, any> | undefined)?.conditions;
  if (!Array.isArray(conditions)) return [];
  return conditions
    .map((condition: any) => condition?.[key])
    .filter((port: unknown): port is string => typeof port === "string");
};

/**
 * Counts references to the legacy "default" port; with apply=true also
 * rewrites them — call inside a transaction when applying.
 *
 * Only action-definition ports are rewritten (node data.inputs/outputs and
 * the edge handles referencing them) — these are what the rename actually
 * broke. "default" stays a legal name everywhere else: routingPort, subworkflow
 * pseudo ports and their edge handles (the engine wires routers by string
 * equality between routingPort and the parent edge port, per dag_schemas.rs),
 * condition ports, and InputRouter/OutputRouter nodes (their canvas handles
 * are frontend-only, so existing routers keep "default").
 */
export function scanLegacyPorts(
  yWorkflows: Y.Map<YWorkflow>,
  apply: boolean,
): number {
  let count = 0;

  yWorkflows.forEach((yWorkflow) => {
    const yNodes = yWorkflow.get("nodes");

    // Per-node context so edges referencing user-named ports are preserved.
    const nodeTypes = new Map<string, string>();
    const routerNodeIds = new Set<string>();
    const customInputPorts = new Map<string, Set<string>>();
    const customOutputPorts = new Map<string, Set<string>>();

    if (yNodes instanceof Y.Map) {
      yNodes.forEach((yNode, nodeId) => {
        nodeTypes.set(nodeId, String((yNode as Y.Map<unknown>).get("type")));
        const yData = (yNode as Y.Map<unknown>).get("data");
        if (!(yData instanceof Y.Map)) return;
        const officialName = String(yData.get("officialName"));
        if (
          officialName === "InputRouter" ||
          officialName === "OutputRouter" ||
          officialName === "Input Router" ||
          officialName === "Output Router"
        ) {
          routerNodeIds.add(nodeId);
        }
        const params = yData.get("params");
        customInputPorts.set(
          nodeId,
          new Set(getConditionPorts(params, "inputPort")),
        );
        customOutputPorts.set(
          nodeId,
          new Set(getConditionPorts(params, "outputPort")),
        );
      });

      yNodes.forEach((yNode, nodeId) => {
        if (routerNodeIds.has(nodeId)) return;
        const yData = (yNode as Y.Map<unknown>).get("data");
        if (!(yData instanceof Y.Map)) return;

        for (const key of ["inputs", "outputs"]) {
          const yPorts = yData.get(key);
          if (!(yPorts instanceof Y.Array)) continue;
          for (let i = 0; i < yPorts.length; i++) {
            if (String(yPorts.get(i)) !== LEGACY_PORT) continue;
            count++;
            if (apply) {
              yPorts.delete(i, 1);
              yPorts.insert(i, [new Y.Text(DEFAULT_EDGE_PORT)]);
            }
          }
        }
      });
    }

    const yEdges = yWorkflow.get("edges");
    if (yEdges instanceof Y.Map) {
      const edgeEnds = [
        {
          handleKey: "sourceHandle",
          nodeKey: "source",
          customPorts: customOutputPorts,
        },
        {
          handleKey: "targetHandle",
          nodeKey: "target",
          customPorts: customInputPorts,
        },
      ];

      yEdges.forEach((yEdge) => {
        for (const { handleKey, nodeKey, customPorts } of edgeEnds) {
          const handle = (yEdge as Y.Map<unknown>).get(handleKey);
          if (handle === undefined || String(handle) !== LEGACY_PORT) continue;

          const nodeId = String((yEdge as Y.Map<unknown>).get(nodeKey));
          // Subworkflow handles are pseudo ports named after routingPorts —
          // user space, not action-definition ports.
          if (nodeTypes.get(nodeId) === "subworkflow") continue;

          // Existing routers keep their "default" handles, so their edges
          // must too.
          if (routerNodeIds.has(nodeId)) continue;

          if (customPorts.get(nodeId)?.has(LEGACY_PORT)) continue;

          count++;
          if (apply)
            (yEdge as Y.Map<unknown>).set(
              handleKey,
              new Y.Text(DEFAULT_EDGE_PORT),
            );
        }
      });
    }
  });

  return count;
}

export const hasLegacyPorts = (yWorkflows: Y.Map<YWorkflow>): boolean =>
  scanLegacyPorts(yWorkflows, false) > 0;

export const migrateLegacyPorts = (yWorkflows: Y.Map<YWorkflow>): number =>
  scanLegacyPorts(yWorkflows, true);
