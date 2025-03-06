import { X } from "@phosphor-icons/react";

import { Input } from "@flow/components";
import { useT } from "@flow/lib/i18n";

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
  editId,
  id,
  name,
  setName,
  onWorkflowChange,
  onWorkflowClose,
  onDoubleClick,
  onSubmit,
}) => {
  const t = useT();
  const isEditing = editId === id;

  return (
    <div
      className={`relative flex h-4/5 w-[150px] shrink-0 items-center justify-center rounded ${currentWorkflowId === id ? "bg-node-entrance/70 hover:bg-node-entrance/80" : "bg-node-entrance/20 hover:bg-node-entrance/30"} group cursor-pointer`}
      onClick={() => onWorkflowChange(id)}
      onDoubleClick={() => onDoubleClick?.(id, name)}
      key={id}>
      {isEditing ? (
        <Input
          className="h-4 border-none text-center text-xs focus-visible:ring-0 dark:font-extralight"
          defaultValue={name}
          placeholder={t("Set Workflow name")}
          autoFocus
          onKeyDownCapture={(e) => e.key === "Enter" && onSubmit()}
          onChange={(e) => setName(e.target.value)}
          onBlur={onSubmit}
        />
      ) : (
        <p
          className={`ml-[15px] mr-[19px] select-none truncate text-center text-xs group-hover:text-white dark:font-extralight ${currentWorkflowId !== id && "text-accent-foreground"}`}>
          {name}
        </p>
      )}
      {!isEditing && (
        <div className="absolute right-0 flex h-full justify-end rounded">
          <div
            className="group flex h-full w-[20px] items-center justify-self-end overflow-hidden rounded px-1 hover:w-[150px] hover:bg-node-exit hover:transition-all hover:delay-200"
            onClick={onWorkflowClose(id)}>
            <div className="flex-1 overflow-hidden">
              <p className="rounded-l text-center text-xs opacity-0 transition-all delay-200 group-hover:opacity-100 dark:font-extralight">
                {t("Close canvas")}
              </p>
            </div>
            <X className="size-[12px] opacity-0 group-hover:opacity-100" />
          </div>
        </div>
      )}
    </div>
  );
};

export default WorkflowTab;
