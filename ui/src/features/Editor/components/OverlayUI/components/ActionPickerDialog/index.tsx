import { EdgeChange } from "@xyflow/react";
import { memo } from "react";

import { Dialog, DialogContent, DialogTitle, Input } from "@flow/components";
import ActionItem from "@flow/components/ActionItem";
import { useT } from "@flow/lib/i18n";
import type { ActionNodeType, Edge, Node } from "@flow/types";

import ActionFilters from "./ActionFilters";
import ActionPickerDetail from "./ActionPickerDetail";
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

const ActionPickerDialog: React.FC<Props> = ({
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
    currentActionByTypes,
    currentCategories,
    currentTags,
    actionTags,
    handleSearchTerm,
    handleSingleClick,
    handleDoubleClick,
    handleActionTypeToggle,
    handleCategoryToggle,
    handleTagToggle,
    handleClearFilters,
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

  const selectedAction = actionsList?.find((a) => a.name === selected);

  return (
    <Dialog open={!!openedActionType} onOpenChange={(o) => !o && onClose()}>
      <DialogContent
        size="3xl"
        position="top"
        className="flex h-[60vh] flex-col gap-0 overflow-hidden p-0">
        <DialogTitle>{t("Choose Action")}</DialogTitle>

        <div className="flex min-h-0 flex-1 overflow-hidden border-t">
          {/* Left panel — filters + list */}
          <div className="flex w-2/5 min-w-0 flex-col border-r">
            <div className="p-3">
              <ActionFilters
                currentActionByTypes={currentActionByTypes}
                currentCategories={currentCategories}
                currentTags={currentTags}
                actionTypes={actionTypes}
                actionCategories={actionCategories}
                actionTags={actionTags}
                isMainWorkflow={isMainWorkflow}
                onActionTypeToggle={handleActionTypeToggle}
                onCategoryToggle={handleCategoryToggle}
                onTagToggle={handleTagToggle}
                onClearFilters={handleClearFilters}>
                <Input
                  className="mx-auto w-full focus-visible:ring-0"
                  placeholder={t("Search")}
                  autoFocus
                  onChange={(e) => handleSearchTerm(e.target.value)}
                />
              </ActionFilters>
            </div>
            <div
              ref={containerRef}
              className="flex-1 overflow-y-auto px-2 pb-1">
              {actionsList?.map((action, idx) => {
                const isSelected = selected === action.name;
                return (
                  <ActionItem
                    key={action.name}
                    itemRefs={itemRefs}
                    idx={idx}
                    action={action}
                    isSelected={isSelected}
                    actionsList={actionsList}
                    onSingleClick={handleSingleClick}
                    onDoubleClick={handleDoubleClick}
                  />
                );
              })}
            </div>
          </div>
          {/* Right panel — detail */}
          <div className="min-w-0 flex-1 overflow-y-auto">
            <ActionPickerDetail action={selectedAction} />
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default memo(ActionPickerDialog);
