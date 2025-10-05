import { useReactFlow, XYPosition } from "@xyflow/react";
import { useCallback, useEffect, useRef, useState } from "react";

import { useDoubleClick } from "@flow/hooks";
import { useAction } from "@flow/lib/fetch";
import i18n from "@flow/lib/i18n/i18n";
import { buildNewCanvasNode } from "@flow/lib/reactFlow";
import { ActionNodeType, Node } from "@flow/types";
import { getRandomNumberInRange } from "@flow/utils/getRandomNumberInRange";

export default ({
  openedActionType,
  isMainWorkflow,
  onNodesAdd,
  onClose,
}: {
  openedActionType: {
    position: XYPosition;
    nodeType: ActionNodeType;
  };
  isMainWorkflow: boolean;
  onNodesAdd: (nodes: Node[]) => void;
  onClose: () => void;
}) => {
  const [searchTerm, setSearchTerm] = useState("");
  const [currentActionByType, setCurrentActionByType] =
    useState<ActionNodeType>(openedActionType.nodeType);

  const actionTypes: { value: ActionNodeType; label: string }[] = [
    { value: "reader", label: "Reader" },
    { value: "transformer", label: "Transformer" },
    { value: "writer", label: "Writer" },
  ];

  const containerRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLDivElement | null)[]>([]);
  // const { handleNodeDropInBatch } = useBatch();
  const { screenToFlowPosition } = useReactFlow();
  const { useGetActionsSegregated } = useAction(i18n.language);
  const { actions } = useGetActionsSegregated({
    isMainWorkflow,
    searchTerm,
    type: currentActionByType,
  });

  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [selected, setSelected] = useState<string | undefined>();

  useEffect(() => {
    if (actions?.length) {
      const actionsList = actions.byType[currentActionByType];

      setSelected(actionsList?.[selectedIndex]?.name ?? "");
    }
  }, [selectedIndex, actions, currentActionByType]);

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
      const newNode = await buildNewCanvasNode({
        position:
          openedActionType.position.x === 0 && openedActionType.position.y === 0
            ? screenToFlowPosition({
                x: window.innerWidth / 2 + randomX,
                y: window.innerHeight / 2 - randomY,
              })
            : openedActionType.position,
        type: name,
      });
      if (!newNode) return;
      onNodesAdd([newNode]);
      // TODO - add drop in batch support
      // onNodesChange(handleNodeDropInBatch(newNode, newNodes));
      onClose();
    },
  );

  const actionsList = actions?.byType[currentActionByType] || [];

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

      const currentActionsList = actions?.byType[currentActionByType] || [];

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
    currentActionByType,
    selectedIndex,
    selected,
    handleDoubleClick,
    setSelectedIndex,
    setSelected,
  ]);

  const handleActionByTypeChange = useCallback(
    (actionByType: ActionNodeType) => {
      setCurrentActionByType(actionByType);
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
