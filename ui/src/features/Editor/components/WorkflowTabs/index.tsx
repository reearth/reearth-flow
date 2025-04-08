import { memo, useState } from "react";

import { ScrollArea } from "@flow/components";
import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { Workflow } from "@flow/types";

import WorkflowTab from "./WorkflowTab";

type Props = {
  currentWorkflowId: string;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
  onWorkflowRename?: (id: string, name: string) => void;
};

const WorkflowTabs: React.FC<Props> = ({
  currentWorkflowId,
  openWorkflows,
  onWorkflowClose,
  onWorkflowChange,
  onWorkflowRename,
}) => {
  const t = useT();
  const { toast } = useToast();

  const [name, setName] = useState<string | undefined>();
  const [editId, setEditId] = useState<string | undefined>();

  const mainWorkflow = openWorkflows?.[0];

  const subWorkflows: Workflow[] | undefined = openWorkflows?.slice(1);

  const handleWorkflowClose =
    (workflowId: string) =>
    (e: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
      e.stopPropagation();
      onWorkflowClose(workflowId);
    };

  const handleDoubleClick = onWorkflowRename
    ? (workflowId: string, name: string | undefined) => {
        setEditId(workflowId);
        setName(name);
      }
    : undefined;

  const handleSubmit = () => {
    if (!name || !editId) {
      setEditId(undefined);
      setName(undefined);
      return;
    }
    const trimmedName = name?.trim();
    if (!trimmedName || trimmedName.length < 1) return;

    try {
      onWorkflowRename?.(editId, trimmedName);
    } catch {
      toast({
        title: t("Unable to rename workflow"),
        description: t("Renaming workflow failed. Please try again later."),
        variant: "destructive",
      });
    }
    setEditId(undefined);
    setName(undefined);
  };

  return (
    <div className="flex gap-1 h-full flex-1 items-end w-full overflow-hidden">
      <div
        className={`rounded-t group flex h-4/5 w-[135px] shrink-0 cursor-pointer items-center justify-center px-[6px]  ${currentWorkflowId === mainWorkflow?.id ? "bg-card" : "bg-card/70 hover:bg-card"}`}
        onClick={() => onWorkflowChange(mainWorkflow?.id)}>
        <p
          className={`select-none truncate text-center text-xs group-hover:text-white dark:font-extralight ${currentWorkflowId !== mainWorkflow?.id && "text-accent-foreground"}`}>
          {t("Main Workflow")}
        </p>
      </div>
      <ScrollArea className="h-full flex-1">
        <div className="flex gap-1 h-full items-end overflow-auto">
          {subWorkflows &&
            subWorkflows.length > 0 &&
            subWorkflows.map((sw) => (
              <WorkflowTab
                currentWorkflowId={currentWorkflowId}
                editId={editId}
                id={sw.id}
                key={sw.id}
                name={sw.name}
                setName={setName}
                onWorkflowChange={onWorkflowChange}
                onWorkflowClose={handleWorkflowClose}
                onDoubleClick={handleDoubleClick}
                onSubmit={handleSubmit}
              />
            ))}
        </div>
      </ScrollArea>
    </div>
  );
};

export default memo(WorkflowTabs);
