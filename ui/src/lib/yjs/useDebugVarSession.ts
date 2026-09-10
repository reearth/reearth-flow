import { useCallback, useMemo } from "react";
import { useY } from "react-yjs";
import { Doc, Map as YMap } from "yjs";

import type { AnyWorkflowVariable } from "@flow/types";

type DebugVarOverride = { value: any; updatedAt: number };
type DebugVarOverrides = Record<string, DebugVarOverride>;

/** Top-level map, so it stays outside the UndoManager's `workflows` scope. */
const MAP_NAME = "debugRunVariableOverrides";

/**
 * Shared staging values for the next debug run.
 *
 * Everyone with the dialog open is configuring the *same* run, so unlike
 * `paramDrafts` — which merges a private draft per client — this is one shared
 * value per variable, last write wins. Seeing a collaborator's value appear as
 * they type is the point.
 */
export default function useDebugVarSession({
  yDoc,
  baseVariables,
}: {
  yDoc?: Doc | null;
  baseVariables: AnyWorkflowVariable[];
}) {
  const yOverrides = useMemo(
    () => yDoc?.getMap<DebugVarOverride>(MAP_NAME),
    [yDoc],
  );
  const overrides = useY(yOverrides ?? new YMap()) as DebugVarOverrides;

  const variables = useMemo(
    () =>
      baseVariables.map((variable) => {
        const override = overrides[variable.id];
        return override
          ? { ...variable, defaultValue: override.value }
          : variable;
      }),
    [baseVariables, overrides],
  );

  const setVariableValue = useCallback(
    (variableId: string, value: any) => {
      yOverrides?.set(variableId, { value, updatedAt: Date.now() });
    },
    [yOverrides],
  );

  /**
   * Drops every staged value. Called once the run has consumed them, and by the
   * last participant to close the dialog — never by a participant leaving while
   * someone else is still editing, which would wipe their work.
   */
  const clearSession = useCallback(() => {
    if (!yOverrides || !yDoc) return;
    yDoc.transact(() => {
      Array.from(yOverrides.keys()).forEach((key) => yOverrides.delete(key));
    }, "debugVarSession");
  }, [yOverrides, yDoc]);

  const hasOverrides = Object.keys(overrides).length > 0;

  return { variables, setVariableValue, clearSession, hasOverrides };
}
