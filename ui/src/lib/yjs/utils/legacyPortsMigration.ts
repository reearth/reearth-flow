import * as Y from "yjs";

import {
  DEFAULT_EDGE_PORT,
  DEFAULT_ROUTING_PORT,
} from "@flow/global-constants";

import type { YWorkflow } from "../types";

// Port name used before the engine renamed its default port to "features"
// (engine v0.0.429, PR #2236). Projects saved before the rename still
// reference it and can no longer run against current actions.
const LEGACY_PORT = "default";

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
 * Covered locations (these reference each other by string equality, so
 * exact-match replacement keeps them consistent):
 * - node data.inputs / data.outputs port lists (action-definition ports)
 * - node data.params.routingPort (InputRouter / OutputRouter)
 * - subworkflow node pseudoInputs / pseudoOutputs portName
 * - edge sourceHandle / targetHandle — EXCEPT handles that match a
 *   user-defined condition port on the endpoint node (e.g. a FeatureFilter
 *   output the user named "default"): the condition param is user data and
 *   is left alone, so its edge handle must stay "default" to keep matching.
 *
 * Composed pseudo port names (e.g. "MyNode-default") are also left alone:
 * they are opaque matched pairs between a router's routingPort and the
 * parent edge handle, and don't need to equal any action port name.
 */
export function scanLegacyPorts(
  yWorkflows: Y.Map<YWorkflow>,
  apply: boolean,
): number {
  let count = 0;

  yWorkflows.forEach((yWorkflow) => {
    const yNodes = yWorkflow.get("nodes");

    // Per-node user-defined condition ports, so edges into/out of a custom port named "default" are preserved.
    const customInputPorts = new Map<string, Set<string>>();
    const customOutputPorts = new Map<string, Set<string>>();

    if (yNodes instanceof Y.Map) {
      yNodes.forEach((yNode, nodeId) => {
        const yData = (yNode as Y.Map<unknown>).get("data");
        if (!(yData instanceof Y.Map)) return;
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

      yNodes.forEach((yNode) => {
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

        for (const key of ["pseudoInputs", "pseudoOutputs"]) {
          const yPseudoPorts = yData.get(key);
          if (!(yPseudoPorts instanceof Y.Array)) continue;
          yPseudoPorts.forEach((yPseudoPort) => {
            if (!(yPseudoPort instanceof Y.Map)) return;
            if (String(yPseudoPort.get("portName")) !== LEGACY_PORT) return;
            count++;
            if (apply)
              yPseudoPort.set("portName", new Y.Text(DEFAULT_ROUTING_PORT));
          });
        }

        // params is stored as a plain object on the node's data map
        const params = yData.get("params") as Record<string, any> | undefined;
        if (
          params &&
          typeof params === "object" &&
          params.routingPort === LEGACY_PORT
        ) {
          count++;
          if (apply)
            yData.set("params", {
              ...params,
              routingPort: DEFAULT_ROUTING_PORT,
            });
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
