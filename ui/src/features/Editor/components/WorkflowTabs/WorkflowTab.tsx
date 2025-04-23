import { Graph, X } from "@phosphor-icons/react";

type Props = {
  currentWorkflowId?: string;
  id: string;
  name?: string;
  onWorkflowChange: (workflowId?: string) => void;
  onWorkflowClose: (
    workflowId: string,
  ) => (e: React.MouseEvent<HTMLDivElement, MouseEvent>) => void;
};

const WorkflowTab: React.FC<Props> = ({
  currentWorkflowId,
  id,
  name,
  onWorkflowChange,
  onWorkflowClose,
}) => {
  return (
    <div
      className={`relative rounded-t flex h-4/5 w-[150px] gap-1 shrink-0 items-end transition-colors ${currentWorkflowId === id ? "bg-node-subworkflow" : "bg-node-subworkflow/50 hover:bg-node-subworkflow"} group cursor-pointer`}
      onClick={() => onWorkflowChange(id)}
      key={id}>
      <div
        className={`h-full flex gap-2 items-center ml-[8px] group-hover:text-white dark:font-extralight ${currentWorkflowId !== id && "text-accent-foreground"}`}>
        <Graph weight="light" />
      </div>
      <div className="flex justify-center items-center text-center w-[100px] h-full overflow-hidden">
        <p className="select-none truncate text-center text-xs w-full">
          {name}
        </p>
      </div>
      <div className="bg-secondary h-full w-[35px] absolute right-0 flex group-hover:delay-200 group-hover:opacity-100 opacity-0 delay-0 transition-all shadow-[-8px_0_8px_rgba(0,0,0,0.1)]">
        <div className="bg-node-entrance/60 w-full flex items-center justify-center rounded-tr">
          <div
            className="transition-all p-1 rounded hover:bg-node-entrance/40"
            onClick={onWorkflowClose(id)}>
            <X />
          </div>
        </div>
      </div>
    </div>
  );
};

export default WorkflowTab;
