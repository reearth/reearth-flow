import type { AnyWorkflowVariable, TriggerVariableConfig } from "@flow/types";

// The value a variable runs with when the trigger sets none. Mirrors the
// server, which treats a blank string or an empty list as no value at all.
export const isEmptyVariableValue = (value: unknown): boolean =>
  value === undefined ||
  value === null ||
  (typeof value === "string" && value.trim() === "") ||
  (Array.isArray(value) && value.length === 0);

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

// The required variables a run would leave empty: the trigger sets no value
// and the deployment has no default. A cleared field is no value, so the
// default applies, as it does when the trigger is saved.
export const missingRequiredVariables = (
  definitions: AnyWorkflowVariable[],
  values: Record<string, any> | undefined,
): string[] =>
  definitions
    .filter((d) => d.required)
    .filter((d) => isEmptyVariableValue(values?.[d.name] ?? d.defaultValue))
    .map((d) => d.name);

// The trigger's values for variables the deployment no longer has.
export const staleVariableNames = (
  definitions: AnyWorkflowVariable[],
  values: Record<string, any> | undefined,
): string[] => {
  const names = new Set(definitions.map((d) => d.name));
  return Object.keys(values ?? {}).filter((name) => !names.has(name));
};
