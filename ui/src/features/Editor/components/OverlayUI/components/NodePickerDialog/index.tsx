import { Fragment, memo } from "react";

import {
  Dialog,
  DialogContent,
  DialogTitle,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
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
    actionTypes,
    currentActionByType,
    handleSearchTerm,
    handleSingleClick,
    handleDoubleClick,
    handleActionByTypeChange,
  } = useHooks({ openedActionType, isMainWorkflow, onNodesAdd, onClose });

  return (
    <Dialog open={!!openedActionType} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogTitle>{t("Choose action")}</DialogTitle>
        <div className="flex items-center gap-2 p-2">
          <Input
            className="mx-auto w-full focus-visible:ring-0"
            placeholder={t("Search")}
            autoFocus
            onChange={(e) => handleSearchTerm(e.target.value)}
          />
          <Select
            value={currentActionByType}
            disabled={currentActionByType === "transformer" && !isMainWorkflow}
            onValueChange={handleActionByTypeChange}>
            <SelectTrigger className="h-[32px] min-w-[150px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {actionTypes.map((actionType) => (
                <SelectItem key={actionType.value} value={actionType.value}>
                  {actionType.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div ref={containerRef} className="max-h-[50vh] overflow-scroll">
          {actionsList?.map((action, idx) => (
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
