import { createContext, FC, PropsWithChildren, useContext } from "react";

import { NodeChange } from "@flow/types";

export type EditorContextType = {
  onNodesChange?: (changes: NodeChange[]) => void;
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
