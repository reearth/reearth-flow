import { CornersIn, CornersOut } from "@phosphor-icons/react";
import { memo } from "react";

import { IconButton } from "@flow/components";

import { WorkflowTabs } from "..";

import { BaseActionButtons } from "./components";
import { Contents } from "./components/Contents";
import useHooks from "./hooks";

type Props = {
  currentWorkflowId?: string;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  isOpen: boolean;
  onOpen: (panel?: "left" | "right" | "bottom") => void;
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowAdd: () => void;
  onWorkflowChange: (workflowId?: string) => void;
};

const BottomPanel: React.FC<Props> = ({
  currentWorkflowId,
  openWorkflows,
  isOpen,
  onOpen,
  onWorkflowClose,
  onWorkflowAdd,
  onWorkflowChange,
}) => {
  const {
    selectedId,
    windowSize,
    panelContentOptions,
    setWindowSize,
    handleSelection,
  } = useHooks({
    isOpen,
    onOpen,
  });

  return (
    <div
      className="box-content flex flex-col justify-end border-t bg-secondary backdrop-blur-md duration-300 ease-in-out"
      style={{
        height: isOpen
          ? windowSize === "max"
            ? "calc(100vh - 1px)"
            : "50vh"
          : "29px",
      }}>
      {isOpen && (
        <div
          id="top-edge"
          className="flex h-[29px] shrink-0 items-center justify-end gap-2">
          <BaseActionButtons
            panelContentOptions={panelContentOptions}
            selectedId={selectedId}
            onSelection={handleSelection}
          />
          {isOpen && (
            <div className="flex h-[29px] items-center px-1">
              <IconButton
                className="h-4/5 w-[55px]"
                icon={windowSize === "min" ? <CornersOut /> : <CornersIn />}
                tooltipPosition="top"
                onClick={() =>
                  windowSize === "min"
                    ? setWindowSize("max")
                    : setWindowSize("min")
                }
              />
            </div>
          )}
        </div>
      )}
      <Contents isOpen={isOpen} selectedId={selectedId} />
      <div
        id="bottom-edge"
        className="mx-2 flex h-[29px] shrink-0 items-center justify-between gap-2 bg-secondary">
        <WorkflowTabs
          currentWorkflowId={currentWorkflowId}
          openWorkflows={openWorkflows}
          onWorkflowClose={onWorkflowClose}
          onWorkflowAdd={onWorkflowAdd}
          onWorkflowChange={onWorkflowChange}
        />
        <div className="flex h-full items-center gap-1">
          {!isOpen && (
            <BaseActionButtons
              panelContentOptions={panelContentOptions}
              selectedId={selectedId}
              onSelection={handleSelection}
            />
          )}
        </div>
      </div>
    </div>
  );
};

export default memo(BottomPanel);
