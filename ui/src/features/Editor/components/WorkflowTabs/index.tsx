import { Plus, X } from "@phosphor-icons/react";
import { memo } from "react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Workflow } from "@flow/types";

type Props = {
  currentWorkflowId?: string;
  workflows: {
    id: string;
    name: string;
  }[];
  onWorkflowChange: (workflowId?: string) => void;
  onWorkflowAdd: () => void;
  onWorkflowRemove: (workflowId: string) => void;
};

const WorkflowTabs: React.FC<Props> = ({
  currentWorkflowId,
  workflows,
  onWorkflowAdd,
  onWorkflowRemove,
  onWorkflowChange,
}) => {
  const t = useT();

  const mainWorkflow = workflows?.[0];

  const subWorkflows: Workflow[] | undefined = workflows?.slice(1);

  const handleWorkflowRemove =
    (workflowId: string) => (e: React.MouseEvent<SVGSVGElement, MouseEvent>) => {
      e.stopPropagation();
      onWorkflowRemove(workflowId);
    };

  return (
    <div className="w-[75vw] bg-zinc-800">
      <div className="flex h-[29px] flex-1 items-center bg-zinc-900/50">
        <div
          className={`mx-1 flex w-28 cursor-pointer items-center justify-center rounded px-[6px] py-[2px] text-zinc-400 ${currentWorkflowId === mainWorkflow?.id ? "bg-zinc-700 text-zinc-300" : "hover:bg-zinc-600"}`}
          onClick={() => onWorkflowChange(mainWorkflow?.id)}>
          <p
            className={`truncate text-center text-xs font-extralight ${currentWorkflowId === mainWorkflow?.id && "text-zinc-300"}`}>
            {t("Main Workflow")}
          </p>
        </div>
        <div className="flex h-[29px] items-center gap-1 overflow-auto">
          {subWorkflows &&
            subWorkflows.length > 0 &&
            subWorkflows.map(sw => (
              <div
                key={sw.id}
                className={`relative flex w-28 items-center justify-center rounded px-[6px] py-[2px] text-zinc-400 transition-colors ${currentWorkflowId === sw?.id ? "bg-zinc-700" : "hover:bg-zinc-600 hover:text-zinc-300"} group cursor-pointer`}
                onClick={() => onWorkflowChange(sw.id)}>
                <X
                  className="absolute right-[2px] hidden size-[15px] group-hover:block group-hover:bg-zinc-600"
                  onClick={handleWorkflowRemove(sw.id)}
                />
                <p
                  className={`truncate text-center text-xs font-extralight group-hover:text-zinc-300 ${currentWorkflowId === sw?.id && "text-zinc-300"}`}>
                  {sw.name}
                </p>
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
