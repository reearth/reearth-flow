import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { cn } from "@flow/lib/utils";
import type { Action } from "@flow/types";
import { typeColorClass, getNodeIcon } from "@flow/utils";

type Props = {
  action?: Action;
  onAdd?: (name: string) => void;
};

const ActionDetails = ({ action, onAdd }: Props) => {
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
    <div className="mx-2 mb-2 flex flex-col gap-4 rounded-xl border border-primary bg-secondary p-4">
      <div className="flex items-center gap-3">
        <div
          className={cn("shrink-0 rounded p-1.5", typeColorClass(action.type))}>
          <Icon size={20} weight="thin" className="text-white" />
        </div>
        <h2 className="text-lg font-semibold">{action.name}</h2>
      </div>

      <div className="flex flex-col flex-wrap gap-1.5">
        <p className="items-center text-xs font-semibold tracking-wide text-muted-foreground">
          {t("Type")}
        </p>
        <div
          className={cn(
            "self-start rounded border p-1 align-middle",
            typeColorClass(action.type),
          )}>
          <p className="self-center text-xs text-zinc-200 capitalize">
            {action.type}
          </p>
        </div>
      </div>
      <div className="flex flex-col flex-wrap gap-1.5">
        <p className="items-center text-xs font-semibold tracking-wide text-muted-foreground">
          {t("Categories")}
        </p>
        <div className="flex">
          {action.categories.map((c) => (
            <div
              key={c}
              className="w-fit rounded border bg-secondary/80 px-1 py-0.5">
              <p className="self-center text-xs ">{c}</p>
            </div>
          ))}
        </div>
      </div>
      {action.tags && action.tags.length > 0 && (
        <div className="flex flex-col flex-wrap gap-1.5">
          <p className="items-center text-xs font-semibold tracking-wide text-muted-foreground">
            {t("Tags")}
          </p>
          <div className="flex gap-1">
            {action.tags.map((tag) => (
              <div
                key={tag}
                className="w-fit rounded border bg-secondary/80 px-1 py-0.5">
                <p className="self-center text-xs ">{tag}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {action.description && (
        <div>
          <p className="mb-1 text-xs font-semibold tracking-wide text-muted-foreground">
            {t("Description")}
          </p>
          <p className="text-sm leading-relaxed">{action.description}</p>
        </div>
      )}
      {onAdd && (
        <Button className="mt-auto w-full" onClick={() => onAdd(action.name)}>
          {t("Add to canvas")}
        </Button>
      )}
    </div>
  );
};

export { ActionDetails };
