import {
  FunnelSimpleIcon,
  ShareNetworkIcon,
  StackIcon,
} from "@phosphor-icons/react";
import { useState } from "react";

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
  setCurrentActionTypeFilter: (actionType: string) => void;
  setCurrentWorkflowFilter: (workflow: string) => void;
  setSearchTerm: (term: string) => void;
};

const SearchFilters = ({
  searchTerm,
  currentActionTypeFilter,
  currentWorkflowFilter,
  actionTypes,
  workflows,
  setCurrentActionTypeFilter,
  setCurrentWorkflowFilter,
  setSearchTerm,
}: SearchFiltersProps) => {
  const t = useT();
  const [isOpen, setIsOpen] = useState<boolean>(false);

  return (
    <Collapsible
      open={isOpen}
      onOpenChange={setIsOpen}
      className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <Input
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
          <SelectTrigger className="h-[28px] w-full truncate">
            <div className="flex items-center gap-2">
              <ShareNetworkIcon weight="light" size={14} />
              <SelectValue />
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
          <SelectTrigger className="h-[28px] w-full">
            <div className="flex items-center gap-2">
              <StackIcon weight="light" size={14} />
              <SelectValue />
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
