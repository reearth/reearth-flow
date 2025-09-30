import { Fragment, memo } from "react";

import { Dialog, DialogContent, DialogTitle, Input } from "@flow/components";
import ActionItem from "@flow/components/ActionItem";
import { useT } from "@flow/lib/i18n";
import type { ActionNodeType, Node } from "@flow/types";

import useHooks from "./hooks";

export type XYPosition = {
  x: number;
  y: number;
};
type Props = {
  openedActionType: {
    position: XYPosition;
    nodeType: ActionNodeType;
  };
  isMainWorkflow: boolean;
  onNodesAdd: (nodes: Node[]) => void;
  onClose: () => void;
};

const NodePickerDialog: React.FC<Props> = ({
  openedActionType,
  onNodesAdd,
  onClose,
  isMainWorkflow,
}) => {
  const t = useT();

  const {
    actionsList,
    containerRef,
    itemRefs,
    selected,
    handleSearchTerm,
    handleSingleClick,
    handleDoubleClick,
  } = useHooks({ openedActionType, isMainWorkflow, onNodesAdd, onClose });

  return (
    <Dialog open={!!openedActionType} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogTitle>{t("Choose action")}</DialogTitle>
        <Input
          className="mx-auto w-full rounded-none border-x-0 border-t-0 border-zinc-700 bg-secondary focus-visible:ring-0"
          placeholder={t("Search")}
          autoFocus
          onChange={(e) => handleSearchTerm(e.target.value)}
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
