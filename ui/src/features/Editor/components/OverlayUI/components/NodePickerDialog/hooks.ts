import { useReactFlow, XYPosition } from "@xyflow/react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

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
  const lastSearchTerm = useRef("");

  const [searchTerm, setSearchTerm] = useState("");

  const containerRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLDivElement | null)[]>([]);
  // const { handleNodeDropInBatch } = useBatch();
  const { screenToFlowPosition } = useReactFlow();
  const { useGetActionsSegregated } = useAction(i18n.language);
  const { actions } = useGetActionsSegregated({
    isMainWorkflow,
    searchTerm,
    type: openedActionType?.nodeType,
  });

  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [selected, setSelected] = useState<string | undefined>();

  useEffect(() => {
    if (actions?.length) {
      const actionsList = actions.byType[openedActionType.nodeType];
      setSelected(actionsList?.[selectedIndex]?.name ?? "");
    }
  }, [selectedIndex, actions, openedActionType?.nodeType]);

  useEffect(() => {
    if (searchTerm === lastSearchTerm.current) return;
    lastSearchTerm.current = searchTerm;
    setSelectedIndex(-1);
    setSelected(undefined);
  }, [searchTerm, lastSearchTerm]);

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

  const actionsList = useMemo(() => {
    return actions?.byType[openedActionType?.nodeType] || [];
  }, [actions, openedActionType?.nodeType]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
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
            if (actionsList && actionsList[newUpIndex]) {
              setSelected(actionsList[newUpIndex].name);
            }
          }

          break;
        case "ArrowDown":
          {
            e.preventDefault();
            const newDownIndex =
              selectedIndex === (actionsList?.length || 1) - 1
                ? selectedIndex
                : selectedIndex + 1;
            setSelectedIndex(newDownIndex);
            if (actionsList && actionsList[newDownIndex]) {
              setSelected(actionsList[newDownIndex].name);
            }
          }
          break;
      }
    },
    [handleDoubleClick, selected, actionsList, selectedIndex],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [actions, selected, handleKeyDown]);

  return {
    actionsList,
    containerRef,
    itemRefs,
    selected,
    setSearchTerm,
    handleSingleClick,
    handleDoubleClick,
  };
};
