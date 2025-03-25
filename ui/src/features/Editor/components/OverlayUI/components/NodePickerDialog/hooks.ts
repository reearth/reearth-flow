import { useReactFlow, XYPosition } from "@xyflow/react";
import { useEffect, useRef, useState } from "react";

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
  const [searchTerm, setSearchTerm] = useState<string>("");
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

  const [selectedIndex, _setSelectedIndex] = useState(0);
  const [selected, setSelected] = useState<string | undefined>();

  useEffect(() => {
    if (actions?.length) {
      const actionsList = actions.byType[openedActionType.nodeType];
      setSelected(actionsList?.[selectedIndex]?.name ?? "");

      const selectedItem = itemRefs.current[selectedIndex];
      if (selectedItem && containerRef.current) {
        selectedItem.scrollIntoView({
          behavior: "smooth",
          block: "nearest",
        });
      }
    }
  }, [selectedIndex, actions, openedActionType?.nodeType]);

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

  const actionsList = actions?.byType[openedActionType?.nodeType] || [];

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
