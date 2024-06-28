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
    <div className="bg-zinc-800 w-[75vw]">
      <div className="flex flex-1 items-center bg-zinc-900/50 h-[29px]">
        <div
          className={`flex justify-center items-center w-28 mx-1 px-[6px] py-[2px] rounded cursor-pointer text-zinc-400 ${currentWorkflowId === mainWorkflow?.id ? "bg-zinc-700 text-zinc-300" : "hover:bg-zinc-600"}`}
          onClick={() => onWorkflowChange(mainWorkflow?.id)}>
          <p
            className={`text-xs text-center font-extralight truncate ${currentWorkflowId === mainWorkflow?.id && "text-zinc-300"}`}>
            {t("Main Workflow")}
          </p>
        </div>
        <div className="flex items-center gap-1 h-[29px] overflow-auto">
          {subWorkflows &&
            subWorkflows.length > 0 &&
            subWorkflows.map(sw => (
              <div
                key={sw.id}
                className={`flex justify-center items-center relative w-28 px-[6px] py-[2px] rounded transition-colors text-zinc-400 ${currentWorkflowId === sw?.id ? "bg-zinc-700" : "hover:bg-zinc-600 hover:text-zinc-300"} cursor-pointer group`}
                onClick={() => onWorkflowChange(sw.id)}>
                <X
                  className="absolute right-[2px] w-[15px] h-[15px] hidden group-hover:bg-zinc-600 group-hover:block"
                  // onClick={() => onWorkflowRemove(sw.id)}
                />
                <p
                  className={`text-xs text-center font-extralight truncate group-hover:text-zinc-300 ${currentWorkflowId === sw?.id && "text-zinc-300"}`}>
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
