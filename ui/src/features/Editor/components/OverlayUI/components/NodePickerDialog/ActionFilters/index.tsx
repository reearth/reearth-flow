import { ReactNode } from "react";

import { Badge } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { cn } from "@flow/lib/utils";

type Props = {
  children: ReactNode;
  currentActionByTypes: string[];
  currentCategories: string[];
  actionTypes: { value: string; label: string }[];
  actionCategories: { value: string; label: string }[];
  isMainWorkflow: boolean;
  onActionTypeToggle: (value: string) => void;
  onCategoryToggle: (value: string) => void;
  onClearFilters: () => void;
};

const ActionFilters = ({
  children,
  currentActionByTypes,
  currentCategories,
  actionTypes,
  actionCategories,
  isMainWorkflow,
  onActionTypeToggle,
  onCategoryToggle,
  onClearFilters,
}: Props) => {
  const t = useT();
  const hasActiveFilters =
    currentActionByTypes.length > 0 || currentCategories.length > 0;

  return (
    <div className="flex flex-col gap-2">
      {children}
      <div className="flex flex-wrap gap-1.5 border-b pb-2">
        {actionTypes.map(({ value, label }) => {
          const isSelected = currentActionByTypes.includes(value);
          const isDisabled =
            (value === "reader" || value === "writer") && !isMainWorkflow;
          return (
            <Badge
              key={value}
              variant={isSelected ? "default" : "secondary"}
              className={cn(
                "cursor-pointer select-none",
                isDisabled && "pointer-events-none opacity-40",
              )}
              onClick={() => onActionTypeToggle(value)}>
              {label}
            </Badge>
          );
        })}
      </div>
      <div className="flex flex-wrap gap-1.5 border-b pb-2">
        {actionCategories.map(({ value, label }) => {
          const isSelected = currentCategories.includes(value);
          return (
            <Badge
              key={value}
              variant={isSelected ? "default" : "secondary"}
              className="cursor-pointer select-none"
              onClick={() => onCategoryToggle(value)}>
              {label}
            </Badge>
          );
        })}
      </div>
      {hasActiveFilters && (
        <button
          className="self-start text-xs text-muted-foreground underline-offset-2 hover:underline"
          onClick={onClearFilters}>
          {t("Clear filters")}
        </button>
      )}
    </div>
  );
};

export default ActionFilters;
