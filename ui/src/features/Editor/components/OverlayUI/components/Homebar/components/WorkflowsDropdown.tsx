import {
  ArrowElbowDownRightIcon,
  CaretDownIcon,
  CaretRightIcon,
  GraphIcon,
  XIcon,
} from "@phosphor-icons/react";
import { memo, useCallback, useMemo } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import type { OpenWorkflow } from "@flow/lib/yjs/useWorkflowTabs";

type Props = {
  openWorkflows: OpenWorkflow[];
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

  // The open workflows the current one is nested in, outermost first, so the
  // trigger shows where the user is and not just the name of the workflow.
  const currentPath = useMemo(() => {
    const byId = new Map(openWorkflows.map((wf) => [wf.id, wf]));
    const path: OpenWorkflow[] = [];
    let wf = byId.get(currentWorkflowId);
    while (wf && !path.includes(wf)) {
      path.unshift(wf);
      wf = wf.parentId ? byId.get(wf.parentId) : undefined;
    }
    return path;
  }, [openWorkflows, currentWorkflowId]);

  const currentWorkflow = currentPath[currentPath.length - 1];
  const ancestors = currentPath.slice(0, -1);

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
      <DropdownMenuTrigger
        nativeButton={false}
        disabled={noOpenSubworkflows}
        render={
          <div
            title={currentPath.map((wf) => wf.name).join(" › ")}
            className={`flex max-w-[300px] min-w-0 flex-1 cursor-pointer items-center justify-center gap-2 rounded-xl bg-border/20 px-2 py-0.5 dark:bg-primary/70 ${noOpenSubworkflows ? "" : "hover:bg-border/30 dark:hover:bg-primary"}`}>
            <div className="flex min-w-0 items-center gap-1 text-sm font-light italic dark:font-extralight">
              {ancestors.length > 0 && (
                <>
                  <p className="min-w-0 shrink truncate text-muted-foreground">
                    {ancestors.map((wf) => wf.name).join(" › ")}
                  </p>
                  <CaretRightIcon
                    size={10}
                    className="shrink-0 text-muted-foreground"
                  />
                </>
              )}
              <p className="max-w-[160px] shrink-0 truncate pr-px">
                {currentWorkflow?.name || "-"}
              </p>
            </div>
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
          <DropdownMenuItem
            key={wf.id}
            className={`group relative h-6 justify-between p-1 ${wf.id === currentWorkflowId ? "bg-accent" : ""}`}
            onClick={() => onWorkflowChange(wf.id)}>
            <div className="flex max-w-[500px] items-center gap-2">
              {Array.from({ length: Math.max(wf.depth - 1, 0) }, (_, i) => (
                <span key={i} className="w-3 shrink-0" />
              ))}
              {wf.depth > 0 && (
                <ArrowElbowDownRightIcon className="-mr-1 shrink-0 text-muted-foreground" />
              )}
              <GraphIcon className="shrink-0" />
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
