import {
  createContext,
  FC,
  MouseEvent,
  PropsWithChildren,
  useContext,
} from "react";

import type { YWorkflow } from "@flow/lib/yjs/types";
import { NodeChange } from "@flow/types";

export type AwarenessSelection = { color: string; userName: string };
export type AwarenessSelectionsMap = Record<string, AwarenessSelection[]>;

export type EditorContextType = {
  onNodesChange?: (changes: NodeChange[]) => void;
  onNodeSettings?: (_e: MouseEvent | undefined, nodeId: string) => void;
  currentYWorkflow?: YWorkflow;
  undoTrackerActionWrapper?: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
  awarenessSelectionsMap?: AwarenessSelectionsMap;
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

export const useAwarenessNodeSelections = (
  nodeId: string,
): AwarenessSelection[] => {
  const { awarenessSelectionsMap } = useEditorContext();
  return awarenessSelectionsMap?.[nodeId] ?? [];
};
