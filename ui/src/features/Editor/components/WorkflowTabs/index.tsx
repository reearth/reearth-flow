import { Plus, X } from "@phosphor-icons/react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";
import { Workflow } from "@flow/types";

type Props = {
  currentWorkflowId?: string;
  onWorkflowChange: (workflowId?: string) => void;
  // onWorkflowAdd: (projectId?: string) => void;
  // onWorkflowRemove: (workflowId?: string) => void;
};

const WorkflowTabs: React.FC<Props> = ({
  currentWorkflowId,
  // onWorkflowAdd,
  // onWorkflowRemove,
  onWorkflowChange,
}) => {
  const t = useT();

  const [currentProject] = useCurrentProject();

  const mainWorkflow = currentProject?.workflows?.[0];

  const subWorkflows: Workflow[] | undefined = currentProject?.workflows?.slice(1);

  // const handleWorkflowRemove = (workflowId: string) => {
  //   const newSubWorkflows = subWorkflows?.filter(w => w.id !== workflowId);
  //   setSubWorkflows(newSubWorkflows);
  // };

  // const handleWorkflowAdd = () => {
  //   const newWorkflow = generateWorkflows(1)[0];
  //   setSubWorkflows([...(subWorkflows ?? []), newWorkflow]);
  // };

  return (
    <div className="w-[75vw]">
      <div className="flex h-[29px] flex-1 items-center">
        <div
          className={`mx-1 flex w-28 cursor-pointer items-center justify-center rounded px-[6px] py-[2px]  ${currentWorkflowId === mainWorkflow?.id ? "bg-accent text-accent-foreground" : "hover:bg-popover"}`}
          onClick={() => onWorkflowChange(mainWorkflow?.id)}>
          <p
            className={`truncate text-center text-xs font-extralight ${currentWorkflowId === mainWorkflow?.id && "bg-primary/50"}`}>
            {t("Main Workflow")}
          </p>
        </div>
        <div className="flex h-[29px] items-center gap-1 overflow-auto">
          {subWorkflows &&
            subWorkflows.length > 0 &&
            subWorkflows.map(sw => (
              <div
                key={sw.id}
                className={`relative flex w-28 items-center justify-center rounded px-[6px] py-[2px] transition-colors ${currentWorkflowId === sw?.id ? "bg-accent text-accent-foreground" : "hover:bg-popover"} group cursor-pointer`}
                onClick={() => onWorkflowChange(sw.id)}>
                <X
                  className="absolute right-[2px] hidden size-[15px] group-hover:block group-hover:bg-primary/50"
                  // onClick={() => onWorkflowRemove(sw.id)}
                />
                <p
                  className={`truncate text-center text-xs font-extralight group-hover:text-primary/50 ${currentWorkflowId === sw?.id && "text-primary/50"}`}>
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
            // onClick={() => onWorkflowAdd(currentProject?.id)}
          />
        </div>
      </div>
    </div>
  );
};

export { WorkflowTabs };
