import type { AnyWorkflowVariable, TriggerVariableConfig } from "@flow/types";

// The deployment's defaults by variable name.
export const definitionDefaults = (
  definitions: AnyWorkflowVariable[],
): Record<string, any> =>
  Object.fromEntries(definitions.map((d) => [d.name, d.defaultValue ?? null]));

// One row per variable the deployment saved, in its order, holding the
// trigger's value where it sets one; then a row for each value the trigger
// keeps for a variable the deployment no longer has.
export const toTypedMappings = (
  definitions: AnyWorkflowVariable[],
  values: Record<string, any>,
  inferStale: (name: string, value: any) => TriggerVariableConfig,
): TriggerVariableConfig[] => {
  const names = new Set(definitions.map((d) => d.name));
  const rows = definitions.map((d): TriggerVariableConfig => ({
    name: d.name,
    type: d.type,
    config: d.config,
    required: d.required,
    defaultValue: d.name in values ? values[d.name] : (d.defaultValue ?? null),
    deploymentDefault: d.defaultValue ?? null,
  }));
  const stale = Object.entries(values)
    .filter(([name]) => !names.has(name))
    .map(([name, value]) => ({ ...inferStale(name, value), stale: true }));
  return [...rows, ...stale];
};

// The trigger's values for variables the deployment no longer has.
export const staleVariableNames = (
  definitions: AnyWorkflowVariable[],
  values: Record<string, any> | undefined,
): string[] => {
  const names = new Set(definitions.map((d) => d.name));
  return Object.keys(values ?? {}).filter((name) => !names.has(name));
};
