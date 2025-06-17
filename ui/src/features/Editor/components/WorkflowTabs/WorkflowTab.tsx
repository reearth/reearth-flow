import { GraphIcon, XIcon } from "@phosphor-icons/react";

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
      className={`relative flex h-4/5 w-[150px] shrink-0 items-end gap-1 rounded-t transition-colors ${currentWorkflowId === id ? "bg-node-subworkflow" : "bg-node-subworkflow/50 hover:bg-node-subworkflow"} group cursor-pointer`}
      onClick={() => onWorkflowChange(id)}
      key={id}>
      <div
        className={`ml-[8px] flex h-full items-center gap-2 group-hover:text-white dark:font-extralight ${currentWorkflowId !== id && "text-accent-foreground"}`}>
        <GraphIcon weight="light" />
      </div>
      <div className="flex h-full w-[100px] items-center justify-center overflow-hidden text-center">
        <p className="w-full truncate text-center text-xs select-none">
          {name}
        </p>
      </div>
      <div className="absolute right-0 flex h-full w-[35px] bg-secondary opacity-0 shadow-[-8px_0_8px_rgba(0,0,0,0.1)] transition-all delay-0 group-hover:opacity-100 group-hover:delay-200">
        <div className="flex w-full items-center justify-center rounded-tr bg-node-entrance/60">
          <div
            className="rounded p-1 transition-all hover:bg-node-entrance/40"
            onClick={onWorkflowClose(id)}>
            <XIcon />
          </div>
        </div>
      </div>
    </div>
  );
};

export default WorkflowTab;
