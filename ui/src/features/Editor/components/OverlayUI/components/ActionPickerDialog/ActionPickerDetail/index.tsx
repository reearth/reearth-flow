import { useT } from "@flow/lib/i18n";
import { cn } from "@flow/lib/utils";
import type { Action } from "@flow/types";
import { getNodeIcon } from "@flow/utils/getNodeIcon";

import { typeColorClass } from "../utils";

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
          className={cn(
            "self-center rounded border p-1 align-middle",
            typeColorClass(action.type),
          )}>
          <p className="self-center text-xs text-zinc-200 capitalize">
            {action.type}
          </p>
        </div>
      </div>
      <div className="flex flex-col flex-wrap  gap-1.5">
        <p className="items-center text-xs font-semibold tracking-wide text-muted-foreground">
          {t("Categories")}
        </p>
        <div className="flex">
          {action.categories.map((c) => (
            <div key={c} className="w-fit rounded border bg-secondary/80 p-0.5">
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
