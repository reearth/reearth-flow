import type { AnyWorkflowVariable, Variable } from "@flow/types";
import { sameVariableValue } from "@flow/utils";
import type { VariableDefaults } from "@flow/utils";

export type DebugRunVariable = {
  // Unique within the list; names need not be.
  key: string;
  name: string;
  value: unknown;
  // Set when the run used a value other than the variable's default.
  changedFrom?: { defaultValue: unknown };
};

const hasValue = (value: unknown) => value !== null && value !== undefined;

const toRow = (
  key: string,
  name: string,
  value: unknown,
  variableDefault?: { value: unknown },
): DebugRunVariable =>
  variableDefault && !sameVariableValue(variableDefault.value, value)
    ? { key, name, value, changedFrom: { defaultValue: variableDefault.value } }
    : { key, name, value };

// Groups items by name, so a lookup can tell a unique name from a shared one.
const byName = <T extends { name: string }>(items: T[] = []) => {
  const map = new Map<string, T[]>();
  for (const item of items) {
    map.set(item.name, [...(map.get(item.name) ?? []), item]);
  }
  return map;
};

const only = <T>(items?: T[]) => (items?.length === 1 ? items[0] : undefined);

// The variables a debug run used, each marked when it differs from the
// variable's default. Variables without a value are left out, as the server
// does not record them.
//
// The job's recorded values come first: they are what the server ran with,
// and they also reach anyone who joins the run. Runs started before the
// server recorded them fall back to the copy this browser kept.
//
// "Default" means the default when the run started (runDefaults), so editing
// a default afterwards does not mark the run's value as changed. Those are
// kept by id, and only by the browser that started the run; recorded values
// carry only a name, so the local copy links the two. Without them, the
// project's current defaults are the closest there is. A name shared by two
// variables cannot be matched either way, so such a value is never marked.
export const toDebugRunVariables = ({
  recorded,
  local,
  runDefaults,
  defaults,
}: {
  recorded?: Variable[];
  local?: AnyWorkflowVariable[];
  runDefaults?: VariableDefaults;
  defaults?: AnyWorkflowVariable[];
}): DebugRunVariable[] => {
  const defaultsById = new Map(defaults?.map((d) => [d.id, d]));

  const defaultForId = (id: string): { value: unknown } | undefined => {
    if (runDefaults && id in runDefaults) return { value: runDefaults[id] };
    const projectVariable = defaultsById.get(id);
    return projectVariable && { value: projectVariable.defaultValue };
  };

  if (recorded?.length) {
    const localByName = byName(local);
    const defaultsByName = byName(defaults);
    return recorded.flatMap(({ key: name, value }, index) => {
      if (!hasValue(value)) return [];
      const id =
        only(localByName.get(name))?.id ?? only(defaultsByName.get(name))?.id;
      return [
        toRow(
          `${index}-${name}`,
          name,
          value,
          id ? defaultForId(id) : undefined,
        ),
      ];
    });
  }

  return (local ?? [])
    .filter(({ defaultValue }) => hasValue(defaultValue))
    .map(({ id, name, defaultValue }) =>
      toRow(id, name, defaultValue, defaultForId(id)),
    );
};
