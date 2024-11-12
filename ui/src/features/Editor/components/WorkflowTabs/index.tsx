import { Plus, X } from "@phosphor-icons/react";
import { memo } from "react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Workflow } from "@flow/types";

type Props = {
  currentWorkflowId?: string;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
  onWorkflowAdd: () => void;
};

const WorkflowTabs: React.FC<Props> = ({
  currentWorkflowId,
  openWorkflows,
  onWorkflowClose,
  onWorkflowAdd,
  onWorkflowChange,
}) => {
  const t = useT();

  const mainWorkflow = openWorkflows?.[0];

  const subWorkflows: Workflow[] | undefined = openWorkflows?.slice(1);

  const handleWorkflowClose =
    (workflowId: string) =>
    (e: React.MouseEvent<SVGSVGElement, MouseEvent>) => {
      e.stopPropagation();
      onWorkflowClose(workflowId);
    };

  return (
    <div className="w-[75vw]">
      <div className="flex h-[29px] flex-1 items-center gap-1">
        <div
          className={`flex h-4/5 w-[135px] cursor-pointer items-center justify-center rounded px-[6px]  ${currentWorkflowId === mainWorkflow?.id ? "bg-accent text-accent-foreground" : "hover:bg-popover"}`}
          onClick={() => onWorkflowChange(mainWorkflow?.id)}>
          <p
            className={`select-none truncate text-center text-xs dark:font-extralight ${currentWorkflowId === mainWorkflow?.id && "text-accent-foreground"}`}>
            {t("Main Workflow")}
          </p>
        </div>
        <div className="flex h-full items-center gap-1 overflow-auto">
          {subWorkflows &&
            subWorkflows.length > 0 &&
            subWorkflows.map((sw) => (
              <div
                className={`relative flex h-4/5 w-[135px] items-center justify-center rounded ${currentWorkflowId === sw?.id ? "bg-node-entrance/70 text-accent-foreground" : "hover:bg-node-entrance/30"} group cursor-pointer`}
                onClick={() => onWorkflowChange(sw.id)}
                onDoubleClick={() => console.log("Double Click")}>
                <p
                  className={`select-none truncate px-[15px] text-center text-xs group-hover:text-accent-foreground dark:font-extralight ${currentWorkflowId === sw?.id && "text-accent-foreground"}`}>
                  {sw.name}
                </p>
                <X
                  className="absolute right-[4px] hidden size-[12px] hover:bg-accent group-hover:block"
                  weight="bold"
                  onClick={handleWorkflowClose(sw.id)}
                />
              </div>
            ))}
        </div>
        <div className="flex items-center">
          <IconButton
            className="h-[25px]"
            icon={<Plus weight="light" />}
            tooltipText={t("Create new sub workflow")}
            onClick={() => onWorkflowAdd()}
          />
        </div>
      </div>
    </div>
  );
};

export default memo(WorkflowTabs);
