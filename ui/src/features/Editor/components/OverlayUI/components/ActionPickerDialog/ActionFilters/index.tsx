import { DatabaseIcon, DiscIcon, LightningIcon } from "@phosphor-icons/react";
import { ChevronDownIcon, ChevronUpIcon } from "@radix-ui/react-icons";
import { useState } from "react";

import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@flow/components";
import { badgeVariants } from "@flow/components/Badge";
import { useT } from "@flow/lib/i18n";
import { cn } from "@flow/lib/utils";
import { ActionNodeType } from "@flow/types/node";

type Props = {
  currentActionByTypes: ActionNodeType[];
  currentCategories: string[];
  currentTags: string[];
  actionTypes: { value: ActionNodeType; label: string }[];
  actionCategories: { value: string; label: string }[];
  actionTags: { value: string; label: string }[];
  isMainWorkflow: boolean;
  onActionTypeToggle: (value?: ActionNodeType) => void;
  onCategoryToggle: (value?: string) => void;
  onTagToggle: (value?: string) => void;
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
  currentActionByTypes,
  currentCategories,
  currentTags,
  actionTypes,
  actionCategories,
  actionTags,
  isMainWorkflow,
  onActionTypeToggle,
  onCategoryToggle,
  onTagToggle,
}: Props) => {
  const t = useT();
  const [tagsOpen, setTagsOpen] = useState(false);
  console.log("currentActionByTypes", currentActionByTypes);

  return (
    <div data-filter-area className="flex flex-col gap-4">
      <div className="flex flex-col">
        <div className="flex w-full items-center justify-between gap-1">
          <span className="ml-1 text-sm font-medium text-foreground">
            {t("Action Types")}
          </span>
          <button
            key="all"
            type="button"
            aria-pressed={
              currentActionByTypes.length === 0 ||
              currentActionByTypes.length === actionTypes.length
            }
            className={cn(
              badgeVariants({
                variant:
                  currentActionByTypes.length === 0 ||
                  currentActionByTypes.length === 3
                    ? "default"
                    : "secondary",
              }),
              "cursor-pointer select-none disabled:pointer-events-none disabled:opacity-40",
            )}
            onClick={() => onActionTypeToggle()}
            onKeyDown={handleRowArrows}>
            {t("All")}
          </button>
        </div>
        <div className="mt-1.5 flex flex-col flex-wrap gap-1.5">
          {actionTypes.map(({ value, label }) => {
            const isSelected =
              currentActionByTypes.length !== 0 &&
              currentActionByTypes.includes(value);
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
                  "cursor-pointer self-start select-none disabled:pointer-events-none disabled:opacity-40",
                )}
                onClick={() => onActionTypeToggle(value)}
                onKeyDown={handleRowArrows}>
                {value === "reader" ? (
                  <DatabaseIcon size={12} weight="thin" className="mr-1" />
                ) : value === "transformer" ? (
                  <LightningIcon size={12} weight="thin" className="mr-1" />
                ) : value === "writer" ? (
                  <DiscIcon
                    size={12}
                    weight="thin"
                    className="mr-1 rotate-180"
                  />
                ) : null}
                {label}
              </button>
            );
          })}
        </div>
      </div>
      <div className="flex flex-col">
        <div className="border-b pb-2">
          <div className="flex w-full items-center justify-between gap-1">
            <div className="flex w-full items-center gap-1">
              <span className="ml-1 text-sm font-medium text-foreground">
                {t("Categories")}
              </span>
              {currentCategories.length > 0 && (
                <span className="text-xs font-medium text-foreground">
                  ({currentCategories.length})
                </span>
              )}
            </div>
            {currentCategories.length > 0 && (
              <button
                key="clear-categories"
                type="button"
                className="cursor-pointer rounded-full px-2.5 py-0.5 text-xs font-medium text-foreground select-none hover:bg-primary"
                onClick={() => onCategoryToggle()}
                onKeyDown={handleRowArrows}>
                {t("Clear")}
              </button>
            )}
          </div>
        </div>
        <div className="mt-1.5 flex flex-col flex-wrap gap-1.5">
          {actionCategories.map(({ value, label }) => {
            const isSelected = currentCategories.includes(value);
            return (
              <button
                key={value}
                type="button"
                aria-pressed={isSelected}
                className={`cursor-pointer self-start rounded-full px-2.5 py-0.5 text-xs font-medium text-foreground select-none hover:bg-primary ${isSelected ? "bg-primary" : "bg-secondary"}`}
                onClick={() => onCategoryToggle(value)}
                onKeyDown={handleRowArrows}>
                {label}
              </button>
            );
          })}
        </div>
      </div>
      <Collapsible className="flex flex-col" open={tagsOpen}>
        <CollapsibleTrigger
          asChild
          className="border-b pb-2"
          onClick={() => setTagsOpen((o) => !o)}>
          <div className="flex w-full items-center justify-between gap-1 hover:cursor-pointer">
            <div className="flex w-full items-center justify-between gap-1">
              <div className="flex w-full items-center gap-1">
                <span className="ml-1 text-sm font-medium text-foreground">
                  {t("Tags")}
                </span>
                {currentTags.length > 0 && (
                  <span className="text-xs font-medium text-foreground">
                    ({currentTags.length})
                  </span>
                )}
              </div>
              {currentTags.length > 0 && (
                <button
                  key="clear-tags"
                  type="button"
                  className="cursor-pointer rounded-full px-2.5 py-0.5 text-xs font-medium text-foreground select-none hover:bg-primary"
                  onClick={(e) => {
                    e.stopPropagation();
                    onTagToggle();
                  }}
                  onKeyDown={handleRowArrows}>
                  {t("Clear")}
                </button>
              )}
            </div>
            {tagsOpen ? <ChevronUpIcon /> : <ChevronDownIcon />}
          </div>
        </CollapsibleTrigger>
        <CollapsibleContent className="mt-1.5 flex flex-wrap gap-1.5">
          {actionTags.map(({ value, label }) => {
            const isSelected = currentTags.includes(value);
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
                onClick={() => onTagToggle(value)}
                onKeyDown={handleRowArrows}>
                {label}
              </button>
            );
          })}
        </CollapsibleContent>
      </Collapsible>
    </div>
  );
};

export default ActionFilters;
