import { MagnifyingGlassIcon, XIcon } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";
import { NodeChange } from "@xyflow/react";

import { VirtualizedTable } from "@flow/components/visualizations/VirtualizedTable";
import { useT } from "@flow/lib/i18n";
import { Node, Workflow } from "@flow/types";

import SearchFilters from "../SearchFilters";

import useHooks, { SearchNodeResult } from "./hooks";
import { Tooltip, TooltipContent, TooltipTrigger } from "@flow/components";

type SearchPanelProps = {
  showSearchPanel: boolean;
  rawWorkflows: Workflow[];
  currentWorkflowId: string;
  onNodesChange?: (changes: NodeChange<Node>[]) => void;

  onWorkflowOpen: (id: string) => void;
  onShowSearchPanel: (open: boolean) => void;
};

const SearchPanel = ({
  showSearchPanel,
  rawWorkflows,
  currentWorkflowId,
  onNodesChange,
  onWorkflowOpen,
  onShowSearchPanel,
}: SearchPanelProps) => {
  const t = useT();

  const {
    filteredNodes,
    selectedNodeId,
    searchTerm,
    currentActionTypeFilter,
    currentWorkflowFilter,
    actionTypes,
    workflows,
    setSearchTerm,
    setCurrentActionTypeFilter,
    setCurrentWorkflowFilter,
    handleRowClick,
    handleRowDoubleClick,
  } = useHooks({
    rawWorkflows,
    currentWorkflowId,
    onNodesChange,
    onWorkflowOpen,
  });

  const searchNodeColumns: ColumnDef<SearchNodeResult>[] = [
    {
      accessorKey: "displayName",
      header: t("Action Name"),
      cell: ({ row }) => (
        <Tooltip>
          <TooltipTrigger asChild>
            <span className="block max-w-[100px] truncate font-medium">
              {row.original.displayName}
            </span>
          </TooltipTrigger>
          <TooltipContent side="top" align="end" className="bg-primary">
            {row.original.displayName}
          </TooltipContent>
        </Tooltip>
      ),
    },
    {
      accessorKey: "workflowName",
      header: t("Workflow"),
      cell: ({ row }) => (
        <Tooltip>
          <TooltipTrigger asChild>
            <span className="block max-w-[100px] truncate font-medium text-muted-foreground">
              {row.original.workflowName}
            </span>
          </TooltipTrigger>
          <TooltipContent side="top" align="end" className="bg-primary">
            {row.original.workflowName}
          </TooltipContent>
        </Tooltip>
      ),
    },
    {
      accessorKey: "nodeType",
      header: t("Type"),
      cell: ({ row }) => (
        <div
          className={`self-center rounded border text-center ${row.original.nodeType === "transformer" ? "bg-node-transformer/35" : row.original.nodeType === "reader" ? "bg-node-reader/35" : row.original.nodeType === "writer" ? "bg-node-writer/35" : row.original.nodeType === "subworkflow" ? "bg-node-subworkflow/35" : "bg-popover"} p-1 align-middle`}>
          <p className="self-center text-xs text-zinc-200 capitalize">
            {row.original.nodeType}
          </p>
        </div>
      ),
    },
  ];

  return (
    <div
      className={`absolute flex h-[600px] z-50 w-[400px] flex-col rounded-md border border-accent bg-primary/50 p-0 backdrop-blur transition-all duration-150 ease-in-out
      ${showSearchPanel ? "pointer-events-auto scale-100 opacity-100" : "pointer-events-none scale-95 opacity-0"}
      `}>
      <div className="flex h-full min-h-0 flex-col gap-2 p-2">
        <div className="relative flex items-center justify-between">
          <div className="flex items-center gap-2">
            <MagnifyingGlassIcon size={18} weight="light" />
            <span className="dark:font-thin">{t("Search Canvas")}</span>
          </div>
          <XIcon
            className="absolute top-1 right-1 cursor-pointer"
            onClick={() => onShowSearchPanel(false)}
          />
        </div>
        <SearchFilters
          searchTerm={searchTerm}
          currentActionTypeFilter={currentActionTypeFilter}
          currentWorkflowFilter={currentWorkflowFilter}
          actionTypes={actionTypes}
          workflows={workflows}
          setSearchTerm={setSearchTerm}
          setCurrentActionTypeFilter={setCurrentActionTypeFilter}
          setCurrentWorkflowFilter={setCurrentWorkflowFilter}
        />
        <div className="flex min-h-0 flex-1 flex-col">
          <VirtualizedTable
            columns={searchNodeColumns}
            data={filteredNodes}
            searchTerm={searchTerm}
            selectedFeatureId={selectedNodeId}
            onRowClick={handleRowClick}
            onRowDoubleClick={handleRowDoubleClick}
            condensed
            surpressAutoScroll
          />
        </div>
      </div>
    </div>
  );
};

export default SearchPanel;
