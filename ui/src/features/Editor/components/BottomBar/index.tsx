import { memo } from "react";

import { useT } from "@flow/lib/i18n";

import { WorkflowTabs } from "..";

import useHooks from "./hooks";

type Props = {
  currentWorkflowId: string;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
  onWorkflowRename: (id: string, name: string) => void;
};

const BottomBar: React.FC<Props> = ({
  currentWorkflowId,
  openWorkflows,
  onWorkflowClose,
  onWorkflowChange,
  onWorkflowRename,
}) => {
  const t = useT();
  const { jobStatus } = useHooks();

  return (
    <div
      id="bottom-edge"
      className="flex h-[29px] shrink-0 items-center justify-between gap-2 border-t bg-secondary px-1">
      <WorkflowTabs
        currentWorkflowId={currentWorkflowId}
        openWorkflows={openWorkflows}
        onWorkflowClose={onWorkflowClose}
        onWorkflowChange={onWorkflowChange}
        onWorkflowRename={onWorkflowRename}
      />
      <div className="flex items-center justify-center gap-2 self-center border-l bg-secondary px-2">
        <p className="text-xs font-light">{t("Debug Status: ")}</p>
        <p className="text-xs font-thin">{jobStatus ?? t("idle")}</p>
        <div
          className={`${
            jobStatus === "completed"
              ? "bg-success"
              : jobStatus === "running"
                ? "active-node-status"
                : jobStatus === "cancelled"
                  ? "bg-warning"
                  : jobStatus === "failed"
                    ? "bg-destructive"
                    : jobStatus === "queued"
                      ? "queued-node-status"
                      : "bg-primary"
          } size-3 rounded-full`}
        />
      </div>
    </div>
  );
};

export default memo(BottomBar);
