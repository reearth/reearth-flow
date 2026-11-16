import type { AnyWorkflowVariable } from "@flow/types";

// The project's default for each workflow variable, by variable id.
export type VariableDefaults = Record<string, unknown>;

export const sameVariableValue = (a: unknown, b: unknown) =>
  JSON.stringify(a ?? null) === JSON.stringify(b ?? null);

export const toVariableDefaults = (
  variables: AnyWorkflowVariable[],
): VariableDefaults =>
  Object.fromEntries(
    variables.map((variable) => [variable.id, variable.defaultValue]),
  );

// Whether a run's value was set by the user rather than taken from the
// defaults the run started from.
export const isCustomizedValue = (
  variableId: string,
  value: unknown,
  runDefaults: VariableDefaults,
) =>
  !(variableId in runDefaults) ||
  !sameVariableValue(value, runDefaults[variableId]);

// The project's current variables, keeping only the values the user set for
// an earlier run. A value that was the default then follows the project's
// default now, so editing a default reaches the next run.
// Without the earlier run's defaults (a run saved before they were kept),
// every earlier value is kept, as before.
export const carryCustomizedValues = (
  current: AnyWorkflowVariable[],
  previous: AnyWorkflowVariable[],
  previousDefaults?: VariableDefaults,
): AnyWorkflowVariable[] =>
  current.map((variable) => {
    const earlier = previous.find((p) => p.id === variable.id);
    if (!earlier) return variable;
    if (
      previousDefaults &&
      !isCustomizedValue(variable.id, earlier.defaultValue, previousDefaults)
    ) {
      return variable;
    }
    return { ...variable, defaultValue: earlier.defaultValue };
  });
