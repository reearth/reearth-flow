import {
  createContext,
  FC,
  MouseEvent,
  PropsWithChildren,
  useContext,
} from "react";

import type { YWorkflow } from "@flow/lib/yjs/types";
import { NodeChange } from "@flow/types";

export type EditorContextType = {
  onNodesChange?: (changes: NodeChange[]) => void;
  onNodeSettings?: (_e: MouseEvent | undefined, nodeId: string) => void;
  currentYWorkflow?: YWorkflow;
  undoTrackerActionWrapper?: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
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
