import { EdgeChange } from "@xyflow/react";
import { Fragment, memo } from "react";

import { Dialog, DialogContent, DialogTitle, Input } from "@flow/components";
import ActionItem from "@flow/components/ActionItem";
import { useT } from "@flow/lib/i18n";
import type { ActionNodeType, Edge, Node } from "@flow/types";

import ActionFilters from "./ActionFilters";
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
  nodes: Node[];
  selectedNodeIds: string[];
  edges?: Edge[];
  isMainWorkflow: boolean;
  openNodePickerViaShortcut: boolean;
  onNodesAdd: (nodes: Node[]) => void;
  onEdgesAdd?: (edges: Edge[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onClose: () => void;
};

const NodePickerDialog: React.FC<Props> = ({
  openedActionType,
  nodes,
  selectedNodeIds,
  edges,
  openNodePickerViaShortcut,
  onNodesAdd,
  onEdgesAdd,
  onEdgesChange,
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
    actionCategories,
    currentActionByType,
    currentCategory,
    handleSearchTerm,
    handleSingleClick,
    handleDoubleClick,
    handleActionByTypeChange,
    handleCategoryChange,
  } = useHooks({
    openedActionType,
    isMainWorkflow,
    nodes,
    selectedNodeIds,
    edges,
    openNodePickerViaShortcut,
    onNodesAdd,
    onEdgesAdd,
    onEdgesChange,
    onClose,
  });
  return (
    <Dialog open={!!openedActionType} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogTitle>{t("Choose Action")}</DialogTitle>
        <div className="p-2">
          <ActionFilters
            currentActionByType={currentActionByType}
            currentCategory={currentCategory}
            actionTypes={actionTypes}
            actionCategories={actionCategories}
            isMainWorkflow={isMainWorkflow}
            onActionByTypeChange={handleActionByTypeChange}
            onCategoryChange={handleCategoryChange}>
            <Input
              className="mx-auto w-full focus-visible:ring-0"
              placeholder={t("Search")}
              autoFocus
              onChange={(e) => handleSearchTerm(e.target.value)}
            />
          </ActionFilters>
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
