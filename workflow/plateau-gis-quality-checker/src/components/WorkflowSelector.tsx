import { Graph } from "@phosphor-icons/react";

import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "./Dropdown";
import { Label } from "./Label";

export type Workflow = {
  id: string;
  name: string;
};

type Props = {
  workflows?: Workflow[];
  selectedWorkflowId?: string;
  onWorkflowIdSelect: (id: string) => void;
};

const WorkflowSelector: React.FC<Props> = ({ workflows, selectedWorkflowId, onWorkflowIdSelect }) => {
  const selectedWorkflow = workflows?.find((workflow) => workflow.id === selectedWorkflowId);

  const handleWorkflowSelect = (id: string) => {
    const workflow = workflows?.find((workflow) => workflow.id === id);
    if (workflow) {
      onWorkflowIdSelect(workflow.id);
    }
  };
  return (
    <div className="flex flex-col gap-2 font-thin">
      <Label htmlFor="workflow-selection">ワークフローを選択</Label>
      <DropdownMenu>
        <DropdownMenuTrigger>
          <div className="flex cursor-pointer items-center rounded border" onClick={() => handleWorkflowSelect("1")}>
            <div className="flex h-[25px] w-[30px] items-center justify-center border-r">
              <Graph />
            </div>
            {selectedWorkflowId && (
              <div className="truncate pl-4 pr-1">
                <p className="truncate text-xs">{selectedWorkflow?.name ?? "Unknown workflow"}</p>
              </div>
            )}
          </div>
        </DropdownMenuTrigger>
        <DropdownMenuContent className="dark w-[400px] bg-secondary" align="center" sideOffset={0}>
          {workflows?.map((workflow) => (
            <DropdownMenuItem key={workflow.id} onSelect={() => handleWorkflowSelect(workflow.id)}>
              {workflow.name}
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
};

export { WorkflowSelector };
