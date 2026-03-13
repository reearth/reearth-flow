import { createContext, FC, PropsWithChildren, useContext } from "react";

export type AwarenessSelection = { color: string; userName: string };
export type AwarenessSelectionsMap = Record<string, AwarenessSelection[]>;

const AwarenessSelectionsContext = createContext<AwarenessSelectionsMap>({});

export const AwarenessSelectionsProvider: FC<
  PropsWithChildren<{ value: AwarenessSelectionsMap }>
> = ({ children, value }) => (
  <AwarenessSelectionsContext.Provider value={value}>
    {children}
  </AwarenessSelectionsContext.Provider>
);

export const useAwarenessNodeSelections = (
  nodeId: string,
): AwarenessSelection[] => {
  const map = useContext(AwarenessSelectionsContext);
  return map[nodeId] ?? [];
};
