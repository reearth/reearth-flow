import { useReactFlow } from "@xyflow/react";
import { Fragment, memo, useEffect, useRef, useState } from "react";

import { Dialog, DialogContent, DialogTitle, Input } from "@flow/components";
import ActionItem from "@flow/components/ActionItem";
import { useDoubleClick } from "@flow/hooks";
import { useAction } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import i18n from "@flow/lib/i18n/i18n";
import { buildNewCanvasNode } from "@flow/lib/reactFlow";
import type { ActionNodeType, Node } from "@flow/types";
import { getRandomNumberInRange } from "@flow/utils/getRandomNumberInRange";

export type XYPosition = {
  x: number;
  y: number;
};
type Props = {
  openedActionType: {
    position: XYPosition;
    nodeType: ActionNodeType;
  };
  onNodesAdd: (nodes: Node[]) => void;
  onClose: () => void;
  isMainWorkflow: boolean;
};

const NodePickerDialog: React.FC<Props> = ({
  openedActionType,
  onNodesAdd,
  onClose,
  isMainWorkflow,
}) => {
  const t = useT();
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

  return (
    <Dialog open={!!openedActionType} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogTitle>{t("Choose action")}</DialogTitle>
        <Input
          className="mx-auto w-full rounded-none border-x-0 border-t-0 border-zinc-700 bg-secondary focus-visible:ring-0"
          placeholder={t("Search")}
          autoFocus
          onChange={(e) => setSearchTerm(e.target.value)}
        />
        <div ref={containerRef} className="max-h-[50vh] overflow-scroll">
          {actionsList.map((action, idx) => (
            <Fragment key={action.name}>
              <ActionItem
                ref={(el) => {
                  itemRefs.current[idx] = el;
                }}
                className={"m-1"}
                action={action}
                selected={selected === action.name}
                onSingleClick={handleSingleClick}
                onDoubleClick={handleDoubleClick}
              />
              {idx !== actionsList.length - 1 && (
                <div className="mx-1 border-b" />
              )}
            </Fragment>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default memo(NodePickerDialog);
