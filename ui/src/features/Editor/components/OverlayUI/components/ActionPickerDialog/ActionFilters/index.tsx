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

const navigateBadgeRow = (
  e: React.KeyboardEvent<HTMLDivElement>,
  onToggle: () => void,
) => {
  if (e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    onToggle();
    return;
  }
  if (e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
  e.preventDefault();
  const siblings = Array.from(
    e.currentTarget.parentElement?.querySelectorAll<HTMLElement>(
      '[tabindex="0"]',
    ) ?? [],
  );
  const idx = siblings.indexOf(e.currentTarget);
  if (e.key === "ArrowRight") siblings[idx + 1]?.focus();
  else if (e.key === "ArrowLeft") siblings[idx - 1]?.focus();
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
            <Badge
              key={value}
              tabIndex={isDisabled ? -1 : 0}
              role="checkbox"
              aria-checked={isSelected}
              variant={isSelected ? "default" : "secondary"}
              className={cn(
                "cursor-pointer select-none focus:ring-2 focus:ring-ring focus:ring-offset-1 focus:outline-none",
                isDisabled && "pointer-events-none opacity-40",
              )}
              onClick={() => onActionTypeToggle(value)}
              onKeyDown={(e) =>
                navigateBadgeRow(e, () => onActionTypeToggle(value))
              }>
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
              tabIndex={0}
              role="checkbox"
              aria-checked={isSelected}
              variant={isSelected ? "default" : "secondary"}
              className="cursor-pointer select-none focus:ring-2 focus:ring-ring focus:ring-offset-1 focus:outline-none"
              onClick={() => onCategoryToggle(value)}
              onKeyDown={(e) =>
                navigateBadgeRow(e, () => onCategoryToggle(value))
              }>
              {label}
            </Badge>
          );
        })}
      </div>
      {hasActiveFilters && (
        <button
          className="self-start text-xs text-muted-foreground underline-offset-2 hover:underline focus:outline-none focus:underline"
          onClick={onClearFilters}>
          {t("Clear filters")}
        </button>
      )}
    </div>
  );
};

export default ActionFilters;
