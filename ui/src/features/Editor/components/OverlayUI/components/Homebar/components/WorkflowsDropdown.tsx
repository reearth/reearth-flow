import { CaretDownIcon, GraphIcon, XIcon } from "@phosphor-icons/react";
import { memo, useCallback, useMemo } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import {
  useEchoDropdown,
  useEchoHover,
} from "@flow/features/Editor/editorContext";
import { ECHO_KEYS, workflowItemEchoKey } from "@flow/lib/yjs";
import { awarenessEchoStyles } from "@flow/utils";

/** Own component so each entry can subscribe to its own echo key. */
const WorkflowMenuItem: React.FC<{
  workflow: { id: string; name: string };
  isMain: boolean;
  onSelect: (workflowId: string) => void;
  onClose: (
    workflowId: string,
  ) => (e: React.MouseEvent<HTMLDivElement, MouseEvent>) => void;
}> = ({ workflow, isMain, onSelect, onClose }) => {
  const { users, hoverProps } = useEchoHover(workflowItemEchoKey(workflow.id));

  return (
    <DropdownMenuItem
      className="group relative h-6 justify-between p-1"
      style={awarenessEchoStyles(users)}
      onClick={() => onSelect(workflow.id)}
      {...hoverProps}>
      <div className="flex max-w-[500px] items-center gap-2">
        <GraphIcon />
        <p className="truncate">{workflow.name}</p>
      </div>
      <div className="flex items-center gap-1">
        {users.length > 0 && (
          <div className="flex items-center -space-x-2">
            {users.slice(0, 3).map((user) => (
              <div
                key={user.userName}
                title={user.userName}
                className="size-3 rounded-full ring-2 ring-secondary/20"
                style={{ backgroundColor: user.color }}
              />
            ))}
          </div>
        )}
        {!isMain && (
          <div
            className="invisible h-4 w-4 group-hover:visible"
            onClick={onClose(workflow.id)}>
            <XIcon />
          </div>
        )}
      </div>
    </DropdownMenuItem>
  );
};

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

  const echoDropdown = useEchoDropdown(ECHO_KEYS.workflowsDropdown);

  return (
    <DropdownMenu {...echoDropdown}>
      <DropdownMenuTrigger
        nativeButton={false}
        disabled={noOpenSubworkflows}
        render={
          <div
            className={`flex max-w-[300px] flex-1 cursor-pointer items-center justify-center gap-2 rounded-xl bg-border/20 px-2 py-0.5 dark:bg-primary/70 ${noOpenSubworkflows ? "" : "hover:bg-border/30 dark:hover:bg-primary"}`}>
            <p className="truncate pr-px text-sm font-light italic dark:font-extralight">
              {currentWorkflow?.name || "-"}
            </p>
            {!noOpenSubworkflows && (
              <div>
                <CaretDownIcon size={12} />
              </div>
            )}
          </div>
        }
      />
      <DropdownMenuContent
        className="min-w-[200px]"
        side="bottom"
        align="center">
        {openWorkflows.map((wf) => (
          <WorkflowMenuItem
            key={wf.id}
            workflow={wf}
            isMain={isMainWorkflow(wf.id)}
            onSelect={onWorkflowChange}
            onClose={handleWorkflowClose}
          />
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export default memo(WorkflowsDropdown);
