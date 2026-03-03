import {
  FunnelSimpleIcon,
  ShareNetworkIcon,
  StackIcon,
} from "@phosphor-icons/react";
import { KeyboardEvent, RefObject, useCallback, useState } from "react";

import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  IconButton,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type SearchFiltersProps = {
  searchTerm: string;
  searchInputRef: RefObject<HTMLInputElement | null>;
  currentActionTypeFilter: string;
  currentWorkflowFilter: string;
  actionTypes: {
    value: string;
    label: string;
  }[];
  workflows: {
    value: string;
    label: string;
  }[];
  onShowSearchPanel: (open: boolean) => void;
  setCurrentActionTypeFilter: (actionType: string) => void;
  setCurrentWorkflowFilter: (workflow: string) => void;
  setSearchTerm: (term: string) => void;
};

const SearchFilters = ({
  searchTerm,
  searchInputRef,
  currentActionTypeFilter,
  currentWorkflowFilter,
  actionTypes,
  workflows,
  onShowSearchPanel,
  setCurrentActionTypeFilter,
  setCurrentWorkflowFilter,
  setSearchTerm,
}: SearchFiltersProps) => {
  const t = useT();
  const [isOpen, setIsOpen] = useState<boolean>(false);

  const handleKeyDown = useCallback(
    (event: KeyboardEvent<HTMLInputElement>) => {
      const isModifierPressed = event.metaKey || event.ctrlKey;
      const isKeyK = event.key === "K" || event.key === "k";
      if (isModifierPressed && isKeyK) {
        onShowSearchPanel(false);
        searchInputRef.current?.blur();
      }
    },
    [onShowSearchPanel, searchInputRef],
  );

  return (
    <Collapsible
      open={isOpen}
      onOpenChange={setIsOpen}
      className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <Input
          ref={searchInputRef}
          onKeyDown={handleKeyDown}
          placeholder={t("Search") + "..."}
          value={searchTerm ?? ""}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="h-[36px]"
        />
        <CollapsibleTrigger asChild>
          <IconButton
            variant="ghost"
            size="icon"
            className="size-8"
            tooltipText={t("Search Filters")}
            icon={<FunnelSimpleIcon size={16} weight="light" />}
          />
        </CollapsibleTrigger>
      </div>
      <CollapsibleContent className="flex gap-2">
        <Select
          value={currentWorkflowFilter}
          onValueChange={setCurrentWorkflowFilter}>
          <SelectTrigger className="h-7 w-full min-w-0">
            <div className="flex min-w-0 flex-1 items-center gap-2">
              <ShareNetworkIcon weight="light" size={14} className="shrink-0" />
              <div className="min-w-0 flex-1 text-left [&>span]:block [&>span]:truncate">
                <SelectValue />
              </div>
            </div>
          </SelectTrigger>
          <SelectContent>
            {workflows.map((option: { value: string; label: string }) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select
          value={currentActionTypeFilter}
          onValueChange={setCurrentActionTypeFilter}>
          <SelectTrigger className="h-7 w-full min-w-0">
            <div className="flex min-w-0 flex-1 items-center gap-2">
              <StackIcon weight="light" size={14} className="shrink-0" />
              <div className="min-w-0 flex-1 text-left [&>span]:block [&>span]:truncate">
                <SelectValue />
              </div>
            </div>
          </SelectTrigger>
          <SelectContent>
            {actionTypes.map((option: { value: string; label: string }) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </CollapsibleContent>
    </Collapsible>
  );
};

export default SearchFilters;
