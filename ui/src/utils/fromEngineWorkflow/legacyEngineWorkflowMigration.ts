import { DEFAULT_EDGE_PORT } from "@flow/global-constants";
import { LEGACY_ACTION_NAMES } from "@flow/lib/yjs/utils/legacyActionNamesMigration";
import type { EngineReadyEdge, EngineReadyWorkflow } from "@flow/types";

// Port name used before the engine renamed its default port to "features"
// (engine v0.0.429, PR #2236).
const LEGACY_PORT = "default";

// A file that references any pre-rename action name must predate the engine
// renames entirely, so its "default" ports are legacy too. Files without
// legacy action names are left alone: their "default" ports (if any) can
// only be deliberate user-chosen names — or the rare pre-rename file built
// solely from unrenamed actions, which the editor's legacy migration dialog
// still catches after import.
export const isLegacyEngineWorkflow = (
  engineWorkflow: EngineReadyWorkflow,
): boolean =>
  engineWorkflow.graphs?.some((graph) =>
    graph.nodes?.some(
      (node) => node.action != null && node.action in LEGACY_ACTION_NAMES,
    ),
  ) ?? false;

const migratePort = (port: string): string =>
  port === LEGACY_PORT ? DEFAULT_EDGE_PORT : port;

const migrateWith = (params: any): any => {
  if (!params || typeof params !== "object") return params;
  const migrated = { ...params };
  if (migrated.routingPort === LEGACY_PORT)
    migrated.routingPort = DEFAULT_EDGE_PORT;
  if (Array.isArray(migrated.conditions)) {
    migrated.conditions = migrated.conditions.map((condition: any) => {
      if (!condition || typeof condition !== "object") return condition;
      const migratedCondition = { ...condition };
      if (migratedCondition.inputPort === LEGACY_PORT)
        migratedCondition.inputPort = DEFAULT_EDGE_PORT;
      if (migratedCondition.outputPort === LEGACY_PORT)
        migratedCondition.outputPort = DEFAULT_EDGE_PORT;
      return migratedCondition;
    });
  }
  return migrated;
};

/**
 * Silently upgrades an imported engine workflow file saved before the engine
 * renames: action names to their space-separated spellings, and the "default"
 * port to "features". Returns the input untouched when the file isn't legacy.
 *
 * Unlike the canvas-level migrations, every port-bearing field is rewritten —
 * edge fromPort/toPort, routingPort, and condition inputPort/outputPort —
 * mirroring how the engine migrated its own workflow fixtures (PR #2236).
 * The engine wires these by string equality, so renaming them all together
 * preserves the wiring; carving out user-named ports here would break it.
 * Node names (user labels) are never touched.
 */
export const migrateLegacyEngineWorkflow = (
  engineWorkflow: EngineReadyWorkflow,
): EngineReadyWorkflow => {
  if (!isLegacyEngineWorkflow(engineWorkflow)) return engineWorkflow;

  return {
    ...engineWorkflow,
    graphs: engineWorkflow.graphs.map((graph) => ({
      ...graph,
      nodes: graph.nodes?.map((node) => ({
        ...node,
        action:
          node.action != null
            ? (LEGACY_ACTION_NAMES[node.action] ?? node.action)
            : node.action,
        with: migrateWith(node.with),
      })),
      edges: graph.edges?.map((edge: EngineReadyEdge) => ({
        ...edge,
        fromPort: migratePort(edge.fromPort),
        toPort: migratePort(edge.toPort),
      })),
    })),
  };
};
