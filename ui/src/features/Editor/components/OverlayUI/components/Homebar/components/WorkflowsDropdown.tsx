import { CaretDownIcon, GraphIcon, XIcon } from "@phosphor-icons/react";
import { memo, useCallback, useMemo } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";

type Props = {
  openWorkflows: {
    id: string;
    name: string;
  }[];
  currentWorkflowId: string;
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
};

const WorkflowsDropdown: React.FC<Props> = ({
  openWorkflows,
  currentWorkflowId,
  onWorkflowChange,
  onWorkflowClose,
}) => {
  const isMainWorkflow = useCallback(
    (id: string) => openWorkflows?.[0]?.id === id,
    [openWorkflows],
  );

  const currentWorkflow = useMemo(
    () => openWorkflows?.find((wf) => wf.id === currentWorkflowId),
    [openWorkflows, currentWorkflowId],
  );

  const handleWorkflowClose = useCallback(
    (workflowId: string) =>
      (e: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
        e.stopPropagation();
        onWorkflowClose(workflowId);
      },
    [onWorkflowClose],
  );

  const noOpenSubworkflows = useMemo(
    () => openWorkflows.length <= 1,
    [openWorkflows],
  );

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild disabled={noOpenSubworkflows}>
        <div
          className={`flex max-w-[300px] flex-1 cursor-pointer items-center justify-center gap-2 rounded-xl bg-primary/70 px-2 py-0.5 ${noOpenSubworkflows ? "" : "hover:bg-primary"}`}>
          <p className="truncate text-sm font-extralight italic">
            {currentWorkflow?.name || "-"}
          </p>
          {!noOpenSubworkflows && (
            <div>
              <CaretDownIcon size={12} />
            </div>
          )}
        </div>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        className="min-w-[200px]"
        side="bottom"
        align="center">
        {openWorkflows.map((wf) => (
          <DropdownMenuItem
            key={wf.id}
            className="group relative h-6 justify-between p-1"
            onClick={() => onWorkflowChange(wf.id)}>
            <div className="flex max-w-[500px] items-center gap-2">
              <GraphIcon />
              <p className="truncate">{wf.name}</p>
            </div>
            {!isMainWorkflow(wf.id) && (
              <div
                className="invisible h-4 w-4 group-hover:visible"
                onClick={handleWorkflowClose(wf.id)}>
                <XIcon />
              </div>
            )}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export default memo(WorkflowsDropdown);
