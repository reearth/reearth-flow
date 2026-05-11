import { FunnelSimpleIcon, StackIcon, TagIcon } from "@phosphor-icons/react";
import { ReactNode, useState } from "react";

import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  IconButton,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  children: ReactNode;
  currentActionByType: string;
  currentCategory: string;
  actionTypes: { value: string; label: string }[];
  actionCategories: { value: string; label: string }[];
  isMainWorkflow: boolean;
  onActionByTypeChange: (value: string) => void;
  onCategoryChange: (value: string) => void;
};

const ActionFilters = ({
  children,
  currentActionByType,
  currentCategory,
  actionTypes,
  actionCategories,
  isMainWorkflow,
  onActionByTypeChange,
  onCategoryChange,
}: Props) => {
  const t = useT();
  const [isOpen, setIsOpen] = useState(false);

  return (
    <Collapsible
      open={isOpen}
      onOpenChange={setIsOpen}
      className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        {children}
        <CollapsibleTrigger asChild>
          <IconButton
            variant="ghost"
            size="icon"
            className="size-8 shrink-0"
            tooltipText={t("Filters")}
            icon={<FunnelSimpleIcon size={16} weight="light" />}
          />
        </CollapsibleTrigger>
      </div>
      <CollapsibleContent className="flex gap-2">
        <Select
          value={currentActionByType}
          onValueChange={onActionByTypeChange}>
          <SelectTrigger className="h-7 w-full min-w-0">
            <div className="flex min-w-0 flex-1 items-center gap-2">
              <StackIcon weight="light" size={14} className="shrink-0" />
              <div className="min-w-0 flex-1 text-left [&>span]:block [&>span]:truncate">
                <SelectValue />
              </div>
            </div>
          </SelectTrigger>
          <SelectContent>
            {actionTypes.map((actionType) => (
              <SelectItem
                key={actionType.value}
                value={actionType.value}
                disabled={
                  (actionType.value === "reader" ||
                    actionType.value === "writer") &&
                  !isMainWorkflow
                }>
                {actionType.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select value={currentCategory} onValueChange={onCategoryChange}>
          <SelectTrigger className="h-7 w-full min-w-0">
            <div className="flex min-w-0 flex-1 items-center gap-2">
              <TagIcon weight="light" size={14} className="shrink-0" />
              <div className="min-w-0 flex-1 text-left [&>span]:block [&>span]:truncate">
                <SelectValue />
              </div>
            </div>
          </SelectTrigger>
          <SelectContent>
            {actionCategories.map((category) => (
              <SelectItem key={category.value} value={category.value}>
                {category.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </CollapsibleContent>
    </Collapsible>
  );
};

export default ActionFilters;
