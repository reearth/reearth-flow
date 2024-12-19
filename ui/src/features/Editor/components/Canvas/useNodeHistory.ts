import { useState, useEffect, useCallback } from "react";

import { Node } from "@flow/types";

type HistoryState = {
  nodes: Node[];
  currentIndex: number;
};

type NavigateNodeHistoryProps = {
  direction: "prev" | "next";
  history: { nodes: Node[]; currentIndex: number };
  setHistory: React.Dispatch<
    React.SetStateAction<{ nodes: Node[]; currentIndex: number }>
  >;
  setSelectedNode: (node: Node) => void;
};

type Props = {
  selected?: Node;
  nodes: Node[];
};

const handleNavigateNodeHistory = ({
  direction,
  history,
  setHistory,
  setSelectedNode,
}: NavigateNodeHistoryProps) => {
  const newIndex =
    direction === "prev" ? history.currentIndex - 1 : history.currentIndex + 1;

  if (newIndex >= 0 && newIndex < history.nodes.length) {
    setHistory((prev) => ({
      ...prev,
      currentIndex: newIndex,
    }));
    setSelectedNode(history.nodes[newIndex]);
  }
};

export const useNodeHistory = ({ selected, nodes }: Props) => {
  const [history, setHistory] = useState<HistoryState>({
    nodes: [],
    currentIndex: -1,
  });
  const [selectedNode, setSelectedNode] = useState<Node | undefined>(selected);

  useEffect(() => {
    if (selected) {
      setHistory((prev) => {
        // Clean the history to only include valid nodes that are in the current list
        const validNodeIds = new Set(nodes.map((node) => node.id));
        const validHistory = prev.nodes.filter((node) =>
          validNodeIds.has(node.id),
        );

        // We only want to add if the node is different and not the same as the last one
        if (validHistory[prev.currentIndex]?.id !== selected.id) {
          const newNodes = [
            ...validHistory.slice(0, prev.currentIndex + 1),
            selected,
          ];
          return {
            nodes: newNodes,
            currentIndex: newNodes.length - 1,
          };
        }
        return { ...prev, nodes: validHistory };
      });
      setSelectedNode(selected);
    }
  }, [selected, nodes]);

  const memoizedNodeHistory = useCallback(
    (direction: "prev" | "next") => {
      handleNavigateNodeHistory({
        direction,
        history,
        setHistory,
        setSelectedNode,
      });
    },
    [history],
  );

  return {
    selectedNode,
    handleNavigateNodeHistory: memoizedNodeHistory,
    nodeHistoryPosition: {
      canGoBack: history.currentIndex > 0,
      canGoForward: history.currentIndex < history.nodes.length - 1,
    },
  };
};
