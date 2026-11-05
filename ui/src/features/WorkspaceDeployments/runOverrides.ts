import type { AnyWorkflowVariable, Variable } from "@flow/types";

// The values the user changed from the deployment's defaults, to send as the
// run's overrides. An untouched variable is left out, so a run with no changes
// sends none and the worker takes the deployed workflow file as it is.
export const toRunOverrides = (
  defaults: AnyWorkflowVariable[],
  edited: AnyWorkflowVariable[],
): Variable[] => {
  const byId = new Map(defaults.map((d) => [d.id, d]));
  return edited
    .filter((v) => {
      const original = byId.get(v.id);
      return (
        !original ||
        JSON.stringify(original.defaultValue ?? null) !==
          JSON.stringify(v.defaultValue ?? null)
      );
    })
    .map((v) => ({ key: v.name, type: v.type, value: v.defaultValue ?? null }));
};
