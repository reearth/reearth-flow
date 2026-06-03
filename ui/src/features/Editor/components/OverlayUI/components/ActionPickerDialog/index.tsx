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
    handleSelectAction,
    handleAddAction,
    handleActionTypeToggle,
    handleCategoryToggle,
    handleTagToggle,
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
        className="flex max-h-[70vh] min-h-[60vh] flex-col gap-0 overflow-hidden p-0">
        <div className="border-b">
          <DialogTitle>{t("Choose Action")}</DialogTitle>
        </div>

        <div className="flex min-h-0 flex-1 overflow-hidden">
          {/* Left panel — filters + list */}
          <div className="flex w-1/4 min-w-0 flex-col overflow-y-auto border-r">
            <div className="flex flex-col gap-2 p-3">
              <Input
                className="mx-auto focus-visible:ring-0"
                placeholder={t("Search Actions")}
                autoFocus
                onChange={(e) => handleSearchTerm(e.target.value)}
              />
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
              />
            </div>
          </div>
          {/* Centre panel — Action List */}
          <div
            ref={containerRef}
            className="flex-1 overflow-y-auto border-r px-2 pt-1 pb-1">
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
                  onSingleClick={handleSelectAction}
                  onDoubleClick={handleAddAction}
                />
              );
            })}
            {actionsList?.length === 0 && (
              <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
                {t("No actions found")}
              </div>
            )}
          </div>
          {/* Right panel — detail */}
          <div className="min-w-0 flex-1 overflow-y-auto">
            <ActionPickerDetail
              action={selectedAction}
              onAdd={handleAddAction}
            />
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default memo(ActionPickerDialog);
