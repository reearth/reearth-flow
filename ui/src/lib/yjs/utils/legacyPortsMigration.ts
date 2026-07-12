import * as Y from "yjs";

import { DEFAULT_EDGE_PORT } from "@flow/global-constants";

import type { YWorkflow } from "../types";

// Port name used before the engine renamed its default port to "features"
// (engine v0.0.429, PR #2236). Projects saved before the rename still
// reference it on action-definition ports and can no longer run.
const LEGACY_PORT = "default";

// Ports a user defined themselves via condition params (e.g. FeatureFilter
// outputPort). These are user data, not action-definition ports — a user may
// legitimately name one "default", so they and their edges must not be
// rewritten.
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
 * Walks every workflow in the doc counting references to the legacy
 * "default" port. With apply=true it also rewrites them to the current
 * port names — call inside a transaction when applying.
 *
 * Only action-definition ports are rewritten — the ones the engine rename
 * actually broke:
 * - node data.inputs / data.outputs port lists (stamped from the action
 *   definition)
 * - edge sourceHandle / targetHandle referencing those ports
 *
 * User-named ports are never touched, so "default" stays a legal name:
 * - params.routingPort, subworkflow pseudoInputs/pseudoOutputs portNames,
 *   and edge handles on subworkflow nodes: the engine wires routers by
 *   string equality between routingPort and the parent edge port
 *   (dag_schemas.rs), so a "default"-named trio still runs — and rewriting
 *   it would rename a port the user may have chosen deliberately.
 * - condition ports (e.g. a FeatureFilter output named "default") and the
 *   edge handles that reference them.
 * - InputRouter / OutputRouter nodes entirely: their canvas handles are
 *   frontend-only, so existing routers keep their "default" handles —
 *   only newly created routers get "features" from the action definition.
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
        if (officialName === "InputRouter" || officialName === "OutputRouter")
          routerNodeIds.add(nodeId);
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
      yEdges.forEach((yEdge) => {
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
