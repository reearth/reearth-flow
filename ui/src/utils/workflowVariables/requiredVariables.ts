import type { AnyWorkflowVariable } from "@flow/types";

// Whether a variable value counts as unset. Mirrors the server, which treats a
// blank string or an empty list as no value at all; 0 and false are values.
export const isEmptyVariableValue = (value: unknown): boolean =>
  value === undefined ||
  value === null ||
  (typeof value === "string" && value.trim() === "") ||
  (Array.isArray(value) && value.length === 0);

// The names of the required variables a run would leave empty. `values` holds
// per-run overrides by name (a trigger's values); without one, or where an
// override is cleared, the variable's own defaultValue applies. A blank
// override does not fall back: it is the value the run would get.
export const missingRequiredVariables = (
  definitions: AnyWorkflowVariable[],
  values?: Record<string, any>,
): string[] =>
  definitions
    .filter((d) => d.required)
    .filter((d) => isEmptyVariableValue(values?.[d.name] ?? d.defaultValue))
    .map((d) => d.name);
