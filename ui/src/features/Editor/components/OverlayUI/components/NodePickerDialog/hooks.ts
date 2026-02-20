import { useReactFlow, XYPosition } from "@xyflow/react";
import { useCallback, useEffect, useRef, useState } from "react";

import { useDoubleClick } from "@flow/hooks";
import { useAction } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import i18n from "@flow/lib/i18n/i18n";
import { buildNewCanvasNode } from "@flow/lib/reactFlow";
import { ActionNodeType, Edge, Node, NodeChange } from "@flow/types";
import { getRandomNumberInRange } from "@flow/utils/getRandomNumberInRange";

type ActionTypeFiltering = "all" | ActionNodeType;
export default ({
  openedActionType,
  isMainWorkflow,
  nodes,
  onNodesAdd,
  onNodesChange,
  onEdgesAdd,
  onClose,
}: {
  openedActionType: {
    position: XYPosition;
    nodeType: ActionNodeType;
  };
  nodes: Node[];
  isMainWorkflow: boolean;
  onNodesAdd: (nodes: Node[]) => void;
  onNodesChange?: (changes: NodeChange[]) => void;
  onEdgesAdd?: (edges: Edge[]) => void;
  onClose: () => void;
}) => {
  const t = useT();
  const [searchTerm, setSearchTerm] = useState("");
  const [currentActionByType, setCurrentActionByType] =
    useState<ActionTypeFiltering>(openedActionType.nodeType);

  const actionTypes: { value: ActionTypeFiltering; label: string }[] = [
    { value: "all", label: t("All Actions") },
    { value: "reader", label: t("Readers") },
    { value: "transformer", label: t("Transformers") },
    { value: "writer", label: t("Writers") },
  ];

  const containerRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLDivElement | null)[]>([]);
  // const { handleNodeDropInBatch } = useBatch();
  const { screenToFlowPosition } = useReactFlow();
  const { useGetActionsSegregated, useGetActions } = useAction(i18n.language);
  const { actions: segregatedActions } = useGetActionsSegregated({
    isMainWorkflow,
    searchTerm,
    type: currentActionByType,
  });

  const { actions } = useGetActions({
    isMainWorkflow,
    searchTerm,
  });

  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [selected, setSelected] = useState<string | undefined>();

  useEffect(() => {
    if (currentActionByType !== "all" && segregatedActions) {
      const actionsList = segregatedActions.byType[currentActionByType];
      setSelected(actionsList?.[selectedIndex]?.name ?? "");
    } else {
      setSelected(actions?.[selectedIndex]?.name ?? "");
    }
  }, [selectedIndex, segregatedActions, actions, currentActionByType]);

  const handleSearchTerm = (newSearchTerm: string) => {
    setSearchTerm(newSearchTerm);
    setSelectedIndex(-1);
    setSelected(undefined);
  };

  useEffect(() => {
    const selectedItem = itemRefs.current[selectedIndex];
    if (selectedItem && containerRef.current) {
      requestAnimationFrame(() => {
        selectedItem.scrollIntoView({
          behavior: "smooth",
          block: "center",
          inline: "nearest",
        });
      });
    }
  }, [selectedIndex]);

  const [handleSingleClick, handleDoubleClick] = useDoubleClick(
    (name?: string) => {
      setSelected((prevName) => (prevName === name ? undefined : name));
    },
    async (name?: string) => {
      if (!name) return;
      // If the position is 0,0 then place it in the center of the screen as this is using shortcut creation and not dnd
      const randomX = getRandomNumberInRange(50, 200);
      const randomY = getRandomNumberInRange(50, 200);
      const selectedNodes = nodes.filter((n) => n.selected);
      const lastSelectedNode = selectedNodes.at(-1);
      let position;
      if (lastSelectedNode) {
        // Move new node to the right of the last selected node
        position = {
          x: lastSelectedNode.position.x + 250, // 250px to the right (adjust as needed)
          y: lastSelectedNode.position.y,
        };
      } else if (
        openedActionType.position.x === 0 &&
        openedActionType.position.y === 0
      ) {
        position = screenToFlowPosition({
          x: window.innerWidth / 2 + randomX,
          y: window.innerHeight / 2 - randomY,
        });
      } else {
        position = openedActionType.position;
      }

      const newNode = await buildNewCanvasNode({
        position,
        type: name,
        lastSelectedNode,
        onEdgesAdd,
      });
      if (!newNode) return;
      if (selectedNodes.length) {
        const nodesToDeselect: NodeChange[] = selectedNodes.map((node) => ({
          type: "select",
          id: node.id,
          selected: false,
        }));
        onNodesChange?.(nodesToDeselect);
      }
      onNodesAdd([newNode]);

      // TODO - add drop in batch support
      // onNodesChange(handleNodeDropInBatch(newNode, newNodes));
      onClose();
    },
  );

  const actionsList =
    currentActionByType !== "all"
      ? segregatedActions?.byType[currentActionByType]
      : actions || [];

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Prevent hotkeys from triggering when Select dropdown is open
      const target = e.target as HTMLElement;
      if (
        target.closest('[role="combobox"]') ||
        target.closest('[role="listbox"]') ||
        target.closest('[role="option"]')
      ) {
        return;
      }

      const currentActionsList =
        currentActionByType !== "all"
          ? segregatedActions?.byType[currentActionByType]
          : actions || [];

      switch (e.key) {
        case "Enter":
          e.preventDefault();
          handleDoubleClick(selected);
          break;
        case "ArrowUp":
          {
            e.preventDefault();
            const newUpIndex =
              selectedIndex === 0 ? selectedIndex : selectedIndex - 1;
            setSelectedIndex(newUpIndex);
            if (currentActionsList && currentActionsList[newUpIndex]) {
              setSelected(currentActionsList[newUpIndex].name);
            }
          }
          break;
        case "ArrowDown":
          {
            e.preventDefault();
            const newDownIndex =
              selectedIndex === (currentActionsList?.length || 1) - 1
                ? selectedIndex
                : selectedIndex + 1;
            setSelectedIndex(newDownIndex);
            if (currentActionsList && currentActionsList[newDownIndex]) {
              setSelected(currentActionsList[newDownIndex].name);
            }
          }
          break;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [
    actions,
    segregatedActions,
    currentActionByType,
    selectedIndex,
    selected,
    handleDoubleClick,
    setSelectedIndex,
    setSelected,
  ]);

  const handleActionByTypeChange = useCallback(
    (actionByType: ActionTypeFiltering) => {
      setCurrentActionByType(actionByType);
      setSelectedIndex(-1);
      setSelected(undefined);
      containerRef.current?.scrollTo({ top: 0, behavior: "smooth" });
    },
    [],
  );

  return {
    actionsList,
    containerRef,
    itemRefs,
    selected,
    currentActionByType,
    actionTypes,
    handleSearchTerm,
    handleSingleClick,
    handleDoubleClick,
    handleActionByTypeChange,
  };
};
