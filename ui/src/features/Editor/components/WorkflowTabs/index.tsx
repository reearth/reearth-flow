import { Plus, X } from "@phosphor-icons/react";
import { TooltipTrigger } from "@radix-ui/react-tooltip";
import { memo, useState } from "react";

import { IconButton, Input, Tooltip, TooltipContent } from "@flow/components";
import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { Workflow } from "@flow/types";

type Props = {
  currentWorkflowId?: string;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
  onWorkflowAdd: () => void;
  onWorkflowRename: (id: string, name: string) => void;
};

const WorkflowTabs: React.FC<Props> = ({
  currentWorkflowId,
  openWorkflows,
  onWorkflowClose,
  onWorkflowAdd,
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
    (e: React.MouseEvent<SVGSVGElement, MouseEvent>) => {
      e.stopPropagation();
      onWorkflowClose(workflowId);
    };

  const handleDoubleClick = (workflowId: string, name: string | undefined) => {
    setEditId(workflowId);
    setName(name);
  };

  const handleSubmit = () => {
    if (!name || !editId) return;
    const trimmedName = name?.trim();
    if (!trimmedName || trimmedName.length < 1) return;

    try {
      onWorkflowRename(editId, trimmedName);
      setEditId(undefined);
      setName(undefined);
    } catch {
      toast({
        title: t("Unable to rename workflow"),
        description: t("Renaming workflow failed. Please try again later."),
        variant: "destructive",
      });
    }
  };

  return (
    <div className="w-[75vw]">
      <div className="flex h-[29px] flex-1 items-center">
        <div
          className={`mx-1 flex w-[135px] cursor-pointer items-center justify-center rounded px-[6px] py-[2px]  ${currentWorkflowId === mainWorkflow?.id ? "bg-accent text-accent-foreground" : "hover:bg-popover"}`}
          onClick={() => onWorkflowChange(mainWorkflow?.id)}>
          <p
            className={`select-none truncate text-center text-xs dark:font-extralight ${currentWorkflowId === mainWorkflow?.id && "text-accent-foreground"}`}>
            {t("Main Workflow")}
          </p>
        </div>
        <div className="flex h-[29px] items-center gap-1 overflow-auto">
          {subWorkflows &&
            subWorkflows.length > 0 &&
            subWorkflows.map((sw) => (
              <Tooltip key={sw.id} delayDuration={1500}>
                <TooltipTrigger asChild>
                  <div
                    className={`relative flex w-[135px] items-center justify-center rounded py-[2px] ${currentWorkflowId === sw?.id ? "bg-node-entrance/70 text-accent-foreground" : "hover:bg-node-entrance/30"} group cursor-pointer`}
                    onClick={() => onWorkflowChange(sw.id)}
                    onDoubleClick={() => handleDoubleClick(sw.id, sw.name)}>
                    {sw.id === editId ? (
                      <Input
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                        onKeyDownCapture={(e) =>
                          e.key === "Enter" && handleSubmit()
                        }
                        placeholder={t("Set Workflow name")}
                        className="h-4 text-xs focus-visible:ring-0"
                        onBlur={handleSubmit}
                      />
                    ) : (
                      <p
                        className={`select-none truncate px-[15px] text-center text-xs group-hover:text-accent-foreground dark:font-extralight ${currentWorkflowId === sw?.id && "text-accent-foreground"}`}>
                        {sw.name}
                      </p>
                    )}

                    <X
                      className="absolute right-[4px] hidden size-[12px] hover:bg-accent group-hover:block"
                      weight="bold"
                      onClick={handleWorkflowClose(sw.id)}
                    />
                  </div>
                </TooltipTrigger>
                <TooltipContent side="top">
                  <p>{sw.name}</p>
                </TooltipContent>
              </Tooltip>
            ))}
        </div>
        <div className="flex items-center">
          <IconButton
            className="h-[25px]"
            icon={<Plus weight="light" />}
            tooltipText={t("Create new sub workflow")}
            onClick={() => onWorkflowAdd()}
          />
        </div>
      </div>
    </div>
  );
};

export default memo(WorkflowTabs);
