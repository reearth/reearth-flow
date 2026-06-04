import {
  ArrowRightIcon,
  DatabaseIcon,
  DiscIcon,
  GraphIcon,
  LightningIcon,
  MagnifyingGlassIcon,
  NoteIcon,
  RectangleDashedIcon,
  XIcon,
} from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";
import { NodeChange } from "@xyflow/react";
import { useEffect, useMemo, useRef } from "react";

import {
  IconButton,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@flow/components";
import { VirtualizedTable } from "@flow/components/visualizations/VirtualizedTable";
import { useT } from "@flow/lib/i18n";
import { Node, Workflow } from "@flow/types";

import SearchFilters from "../SearchFilters";

import useHooks, { SearchNodeResult } from "./hooks";

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
  const searchInputRef = useRef<HTMLInputElement>(null);
  const t = useT();
  const {
    filteredNodes,
    selectedNodeId,
    searchTerm,
    currentActionTypeFilter,
    currentWorkflowFilter,
    actionTypes,
    workflows,
    nodeSearchOptions,
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

  const searchNodeColumns: ColumnDef<SearchNodeResult | undefined, unknown>[] =
    [
      {
        accessorFn: (row) => row?.displayName,
        id: "displayName",
        header: t("Display Name"),
        cell: ({ row }) => (
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex w-[300px] items-center gap-2">
                <div
                  className={`flex w-[24px] justify-center rounded border text-center ${row.original?.nodeType === "transformer" ? "bg-node-transformer/60" : row.original?.nodeType === "reader" ? "bg-node-reader/60" : row.original?.nodeType === "writer" ? "bg-node-writer/60" : row.original?.nodeType === "subworkflow" ? "bg-node-subworkflow/60" : "bg-popover"} p-1 align-middle`}>
                  <p className="self-center text-xs text-zinc-200 capitalize">
                    {row.original?.nodeType === "reader" ? (
                      <DatabaseIcon className="self-center" />
                    ) : row.original?.nodeType === "writer" ? (
                      <DiscIcon className="self-center" />
                    ) : row.original?.nodeType === "subworkflow" ? (
                      <GraphIcon className="self-center" />
                    ) : row.original?.nodeType === "batch" ? (
                      <RectangleDashedIcon className="self-center" />
                    ) : row.original?.nodeType === "note" ? (
                      <NoteIcon className="self-center" />
                    ) : (
                      <LightningIcon className="self-center" />
                    )}
                  </p>
                </div>
                <div className="truncate">
                  <span className="block truncate font-medium">
                    {row.original?.displayName}
                  </span>
                  <span className="block truncate font-medium text-muted-foreground">
                    ({row.original?.officialName})
                  </span>
                </div>
              </div>
            </TooltipTrigger>
            <TooltipContent
              side="right"
              sideOffset={-180}
              className="bg-primary">
              {row.original?.displayName}
            </TooltipContent>
          </Tooltip>
        ),
      },
      {
        accessorFn: (row) => row?.workflowName,
        id: "workflowName",
        header: t("Workflow"),
        cell: ({ row }) => (
          <Tooltip>
            <TooltipTrigger asChild>
              <span className="block max-w-[120px] truncate font-medium text-muted-foreground">
                {row.original?.workflowName}
              </span>
            </TooltipTrigger>
            <TooltipContent
              side="left"
              sideOffset={-100}
              align="center"
              className="bg-primary">
              {row.original?.workflowName}
            </TooltipContent>
          </Tooltip>
        ),
      },
      {
        accessorFn: (row) => row?.workflowName,
        id: "actions",
        header: undefined,
        cell: ({ row }) => (
          <div className="flex justify-end">
            <IconButton
              icon={<ArrowRightIcon />}
              onClick={() => handleRowDoubleClick(row.original)}
            />
          </div>
        ),
      },
    ];

  const selectedRowIndex = useMemo(() => {
    if (!selectedNodeId || !filteredNodes) return -1;

    return filteredNodes.findIndex((row: any) => row.id === selectedNodeId);
  }, [selectedNodeId, filteredNodes]);

  // Focus search input when panel opens
  useEffect(() => {
    if (showSearchPanel && searchInputRef.current) {
      searchInputRef.current.focus();
    }
  }, [showSearchPanel, searchInputRef]);

  return (
    <div
      className={`absolute z-50 flex h-[600px] w-[550px] flex-col rounded-md border border-accent bg-primary/50 p-0 backdrop-blur transition-all duration-150 ease-in-out
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
          searchInputRef={searchInputRef}
          currentActionTypeFilter={currentActionTypeFilter}
          currentWorkflowFilter={currentWorkflowFilter}
          actionTypes={actionTypes}
          workflows={workflows}
          onShowSearchPanel={onShowSearchPanel}
          setSearchTerm={setSearchTerm}
          setCurrentActionTypeFilter={setCurrentActionTypeFilter}
          setCurrentWorkflowFilter={setCurrentWorkflowFilter}
        />
        <div className="flex min-h-0 flex-1 flex-col">
          <VirtualizedTable
            columns={searchNodeColumns}
            data={filteredNodes}
            searchTerm={searchTerm}
            selectedRowIndex={selectedRowIndex}
            onRowClick={handleRowClick}
            onRowDoubleClick={handleRowDoubleClick}
            customGlobalFilterFn={nodeSearchOptions}
          />
        </div>
      </div>
    </div>
  );
};

export default SearchPanel;
