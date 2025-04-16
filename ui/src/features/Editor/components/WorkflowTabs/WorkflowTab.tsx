import { Graph, X } from "@phosphor-icons/react";

type Props = {
  currentWorkflowId?: string;
  editId?: string;
  id: string;
  name?: string;
  setName: (name: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
  onWorkflowClose: (
    workflowId: string,
  ) => (e: React.MouseEvent<HTMLDivElement, MouseEvent>) => void;
  onDoubleClick?: (workflowId: string, name: string | undefined) => void;
  onSubmit: () => void;
};

const WorkflowTab: React.FC<Props> = ({
  currentWorkflowId,
  id,
  name,
  onWorkflowChange,
  onWorkflowClose,
  onDoubleClick,
}) => {
  return (
    <div
      className={`relative rounded-t flex h-4/5 w-[150px] shrink-0 items-end justify-center transition-colors ${currentWorkflowId === id ? "bg-node-subworkflow" : "bg-node-subworkflow/50 hover:bg-node-subworkflow"} group cursor-pointer`}
      onClick={() => onWorkflowChange(id)}
      onDoubleClick={() => onDoubleClick?.(id, name)}
      key={id}>
      <div
        className={`h-full flex gap-2 items-center justify-center ml-[15px] mr-[19px] group-hover:text-white dark:font-extralight ${currentWorkflowId !== id && "text-accent-foreground"}`}>
        <Graph weight="light" />
        <p className="select-none truncate text-center text-xs">{name}</p>
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
