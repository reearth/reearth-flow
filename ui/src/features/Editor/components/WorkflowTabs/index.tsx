import { memo } from "react";

import { ScrollArea } from "@flow/components";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useT } from "@flow/lib/i18n";
import { Workflow } from "@flow/types";

import WorkflowTab from "./WorkflowTab";

type Props = {
  currentWorkflowId: string;
  isMainWorkflow: boolean;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
};

const WorkflowTabs: React.FC<Props> = ({
  currentWorkflowId,
  isMainWorkflow,
  openWorkflows,
  onWorkflowClose,
  onWorkflowChange,
}) => {
  const t = useT();

  const subWorkflows: Workflow[] | undefined = openWorkflows?.slice(1);

  const handleWorkflowClose =
    (workflowId: string) =>
    (e: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
      e.stopPropagation();
      onWorkflowClose(workflowId);
    };

  return (
    <div className="flex h-full w-full flex-1 items-end overflow-hidden">
      <div
        className={`group flex h-4/5 w-[135px] shrink-0 cursor-pointer items-center justify-center rounded-t px-[6px] ${isMainWorkflow ? "border-x border-t bg-card" : "border-b border-node-subworkflow bg-card/70 hover:bg-card"}`}
        onClick={() => onWorkflowChange(DEFAULT_ENTRY_GRAPH_ID)}>
        <p
          className={`truncate text-center text-xs select-none group-hover:text-white dark:font-extralight ${isMainWorkflow && "text-accent-foreground"}`}>
          {t("Main Workflow")}
        </p>
      </div>
      <ScrollArea
        className={`h-full flex-1 border-b pl-1 ${!isMainWorkflow ? "border-node-subworkflow" : ""}`}>
        <div className="flex h-full items-end gap-1 overflow-auto">
          {subWorkflows &&
            subWorkflows.length > 0 &&
            subWorkflows.map((sw) => (
              <WorkflowTab
                currentWorkflowId={currentWorkflowId}
                id={sw.id}
                key={sw.id}
                name={sw.name}
                onWorkflowChange={onWorkflowChange}
                onWorkflowClose={handleWorkflowClose}
              />
            ))}
        </div>
      </ScrollArea>
    </div>
  );
};

export default memo(WorkflowTabs);
