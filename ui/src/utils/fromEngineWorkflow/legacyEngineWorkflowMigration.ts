import { DEFAULT_EDGE_PORT } from "@flow/global-constants";
import { LEGACY_ACTION_NAMES } from "@flow/lib/yjs/utils/legacyActionNamesMigration";
import type { EngineReadyEdge, EngineReadyWorkflow } from "@flow/types";

// Port name used before the engine renamed its default port to "features" (engine PR #2236).
const LEGACY_PORT = "default";

// A file referencing any pre-rename action name must predate the renames
// entirely, so its "default" ports are legacy too. Files without legacy
// action names are left alone — their "default" ports (if any) are deliberate
// user-chosen names.
export const isLegacyEngineWorkflow = (
  engineWorkflow: EngineReadyWorkflow,
): boolean =>
  engineWorkflow.graphs?.some((graph) =>
    graph.nodes?.some(
      (node) =>
        node.action != null &&
        Object.prototype.hasOwnProperty.call(LEGACY_ACTION_NAMES, node.action),
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
 * renames. Returns the input untouched when the file isn't legacy.
 *
 * Unlike the canvas-level migrations, every port-bearing field is rewritten
 * unconditionally (edge fromPort/toPort, routingPort, condition ports) since
 * the engine wires them by string equality — carving out user-named ports
 * here would break the wiring.
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
