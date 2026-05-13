import { ReactNode } from "react";

import { badgeVariants } from "@flow/components/Badge";
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

const handleRowArrows = (e: React.KeyboardEvent<HTMLButtonElement>) => {
  if (e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
  e.preventDefault();
  const siblings = Array.from(
    e.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>(
      "button:not(:disabled)",
    ) ?? [],
  );
  const idx = siblings.indexOf(e.currentTarget);
  if (e.key === "ArrowRight") siblings[idx + 1]?.focus();
  else siblings[idx - 1]?.focus();
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
    <div data-filter-area className="flex flex-col gap-2">
      {children}
      <div className="flex flex-wrap gap-1.5 border-b pb-2">
        {actionTypes.map(({ value, label }) => {
          const isSelected = currentActionByTypes.includes(value);
          const isDisabled =
            (value === "reader" || value === "writer") && !isMainWorkflow;
          return (
            <button
              key={value}
              type="button"
              disabled={isDisabled}
              aria-pressed={isSelected}
              className={cn(
                badgeVariants({
                  variant: isSelected ? "default" : "secondary",
                }),
                "cursor-pointer select-none disabled:pointer-events-none disabled:opacity-40",
              )}
              onClick={() => onActionTypeToggle(value)}
              onKeyDown={handleRowArrows}>
              {label}
            </button>
          );
        })}
      </div>
      <div className="flex flex-wrap gap-1.5 border-b pb-2">
        {actionCategories.map(({ value, label }) => {
          const isSelected = currentCategories.includes(value);
          return (
            <button
              key={value}
              type="button"
              aria-pressed={isSelected}
              className={cn(
                badgeVariants({
                  variant: isSelected ? "default" : "secondary",
                }),
                "cursor-pointer select-none",
              )}
              onClick={() => onCategoryToggle(value)}
              onKeyDown={handleRowArrows}>
              {label}
            </button>
          );
        })}
      </div>
      {hasActiveFilters && (
        <button
          type="button"
          className="self-start text-xs text-muted-foreground underline-offset-2 hover:underline focus:underline focus:outline-none"
          onClick={onClearFilters}>
          {t("Clear filters")}
        </button>
      )}
    </div>
  );
};

export default ActionFilters;
