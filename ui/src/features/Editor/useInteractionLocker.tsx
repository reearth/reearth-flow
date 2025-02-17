import { useReactFlow } from "@xyflow/react";
import React, { createContext, useContext, useState, useCallback } from "react";

import type { Node } from "@flow/types";

type LockerContextType = {
  interactionLockedNodes: Node[];
  lockNodeInteraction: (node: Node) => void;
  unlockNodeInteraction: (node: Node) => void;
  unlockAllNodes: () => void;
};

const LockerContext = createContext<LockerContextType | undefined>(undefined);

export const LockerProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [interactionLockedNodes, setInteractionLockedNodes] = useState<Node[]>(
    [],
  );
  const { fitView } = useReactFlow();

  const lockNodeInteraction = useCallback(
    (node: Node) => {
      setInteractionLockedNodes((currentLockedNodes) => {
        if (currentLockedNodes.some((n) => n.id === node.id))
          return currentLockedNodes;

        const updatedLockedNodes = [...currentLockedNodes, node];
        fitView({
          nodes: [{ id: node.id }],
          duration: 500,
          padding: 2,
        });
        return updatedLockedNodes;
      });
    },
    [fitView],
  );

  const unlockNodeInteraction = useCallback((node: Node) => {
    setInteractionLockedNodes((currentLockedNodes) =>
      currentLockedNodes.filter((n) => n.id !== node.id),
    );
  }, []);

  const unlockAllNodes = useCallback(() => {
    setInteractionLockedNodes([]);
  }, []);

  return (
    <LockerContext.Provider
      value={{
        interactionLockedNodes,
        lockNodeInteraction,
        unlockNodeInteraction,
        unlockAllNodes,
      }}>
      {children}
    </LockerContext.Provider>
  );
};

export const useLocker = () => {
  const context = useContext(LockerContext);
  if (!context) {
    throw new Error("useLocker must be used within a LockerProvider");
  }
  return context;
};
