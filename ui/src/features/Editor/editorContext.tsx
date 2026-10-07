import type { FC, MouseEvent, PropsWithChildren } from "react";
import { createContext, useContext } from "react";
import type { Doc } from "yjs";

import type { YWorkflow } from "@flow/lib/yjs/types";
import type {
  NodeChange,
  AwarenessSelection,
  AwarenessSelectionsMap,
} from "@flow/types";

export type WorkflowVarAwareness = {
  onDialogOpen: () => void;
  onDialogClose: () => void;
  onFieldFocus: (variableId: string | null, field: string | null) => void;
  onEditStart: (variableId: string | null) => void;
};

export type EditorContextType = {
  isLocked: boolean;
  isReaderRestricted: boolean;
  canViewIntermediateData: boolean;
  onNodesChange?: (changes: NodeChange[]) => void;
  onNodeSettings?: (_e: MouseEvent | undefined, nodeId: string) => void;
  currentYWorkflow?: YWorkflow;
  undoTrackerActionWrapper?: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
  awarenessSelectionsMap?: AwarenessSelectionsMap;
  yDoc?: Doc | null;
  workflowVarAwareness?: WorkflowVarAwareness;
  staleNodeIds?: Set<string>;
  /** Every node id across the project's workflows; diagnostics are per-node. */
  workflowNodeIds?: string[];
  /** nodeId -> worst diagnostic severity from the current debug run. */
  diagnosticSeverityByNodeId?: Map<string, string>;
  /**
   * Reveals a node by id: opens its workflow, centres on it and selects it.
   * Panels that only know a node id — the diagnostics table, say — use this to
   * send the user to the action that the row is about.
   */
  onNodeNavigate?: (nodeId: string) => void;
};

const EditorContext = createContext<EditorContextType | undefined>(undefined);

export const EditorProvider: FC<
  PropsWithChildren<{ value: EditorContextType }>
> = ({ children, value }) => (
  <EditorContext.Provider value={value}>{children}</EditorContext.Provider>
);

export const useEditorContext = (): EditorContextType => {
  const ctx = useContext(EditorContext);
  if (!ctx) {
    throw new Error("Could not find EditorProvider");
  }

  return ctx;
};

/**
 * Editing is disabled either because the project is locked or because the
 * current user only has read access.
 */
export const useIsReadOnly = (): boolean => {
  const { isLocked, isReaderRestricted } = useEditorContext();
  return isLocked || isReaderRestricted;
};

export const useAwarenessNodeSelections = (
  nodeId: string,
): AwarenessSelection[] => {
  const { awarenessSelectionsMap } = useEditorContext();
  return awarenessSelectionsMap?.[nodeId] ?? [];
};
