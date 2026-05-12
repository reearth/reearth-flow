import { useT } from "@flow/lib/i18n";
import { cn } from "@flow/lib/utils";
import type { Action } from "@flow/types";
import { getNodeIcon } from "@flow/utils/getNodeIcon";

const typeColorClass = (type: string) => {
  switch (type) {
    case "transformer":
      return "bg-node-transformer/80";
    case "reader":
      return "bg-node-reader/80";
    case "writer":
      return "bg-node-writer/80";
    default:
      return "bg-secondary";
  }
};

type Props = {
  action?: Action;
};

const ActionPickerDetail = ({ action }: Props) => {
  const t = useT();

  if (!action) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
        {t("Select an action to see details")}
      </div>
    );
  }

  const Icon = getNodeIcon(action.type);

  return (
    <div className="flex flex-col gap-4 p-4">
      <div className="flex items-center gap-3">
        <div
          className={cn("shrink-0 rounded p-1.5", typeColorClass(action.type))}>
          <Icon size={20} weight="thin" className="text-white" />
        </div>
        <h2 className="text-lg font-semibold">{action.name}</h2>
      </div>

      <div className="flex flex-wrap gap-1.5">
        <div
          className={`self-center rounded border  ${action.type === "transformer" ? "bg-node-transformer/95 dark:bg-node-transformer/60" : action.type === "reader" ? "bg-node-reader/95 dark:bg-node-reader/60" : action.type === "writer" ? "bg-node-writer/85 dark:bg-node-writer/30" : "bg-popover"} p-0.5 align-middle`}>
          <p className="self-center text-xs capitalize">{action.type}</p>
        </div>
      </div>
      <div className="flex flex-col flex-wrap  gap-1.5">
        <p className="items-center text-xs font-semibold tracking-wide text-muted-foreground">
          {t("Categories")}
        </p>
        <div className="flex">
          {action.categories.map((c) => (
            <div className="w-fit rounded border bg-secondary/80 p-0.5">
              <p className="self-center text-xs ">{c}</p>
            </div>
          ))}
        </div>
      </div>

      {action.description && (
        <div>
          <p className="mb-1 text-xs font-semibold tracking-wide text-muted-foreground">
            {t("Description")}
          </p>
          <p className="text-sm leading-relaxed">{action.description}</p>
        </div>
      )}
    </div>
  );
};

export default ActionPickerDetail;
