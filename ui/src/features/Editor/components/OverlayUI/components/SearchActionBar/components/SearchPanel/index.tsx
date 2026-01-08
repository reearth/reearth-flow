import { MagnifyingGlassIcon, XIcon } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";
import { useState } from "react";

import { VirtualizedTable } from "@flow/components/visualizations/VirtualizedTable";
import { useT } from "@flow/lib/i18n";
import { Workflow } from "@flow/types";

import useHooks, { SearchNodeResult } from "./hooks";

type SearchPanelProps = {
  rawWorkflows: Workflow[];
  currentWorkflowId: string;
  onWorkflowOpen: (id: string) => void;
  onShowSearchPanel: (open: boolean) => void;
};

const SearchPanel = ({
  rawWorkflows,
  currentWorkflowId,
  onWorkflowOpen,
  onShowSearchPanel,
}: SearchPanelProps) => {
  const t = useT();

  const { allNodes, handleNavigateToNode } = useHooks({
    rawWorkflows,
    currentWorkflowId,
    onWorkflowOpen,
  });

  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);

  const handleRowClick = (node: SearchNodeResult) => {
    setSelectedNodeId(node.id);
  };

  const handleRowDoubleClick = (node: SearchNodeResult) => {
    handleNavigateToNode(node);
  };

  const searchNodeColumns: ColumnDef<SearchNodeResult>[] = [
    {
      accessorKey: "displayName",
      header: t("Action Name"),
      cell: ({ row }) => (
        <span className="block max-w-[100px] truncate font-medium">
          {row.original.displayName}
        </span>
      ),
    },
    {
      accessorKey: "workflowName",
      header: t("Workflow"),
      cell: ({ row }) => (
        <span className="block max-w-[100px] truncate font-medium text-muted-foreground">
          {row.original.workflowName}
        </span>
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
      <div className="flex min-h-0 flex-1 flex-col">
        <VirtualizedTable
          columns={searchNodeColumns}
          data={allNodes}
          showFiltering
          selectedFeatureId={selectedNodeId}
          onRowClick={handleRowClick}
          onRowDoubleClick={handleRowDoubleClick}
          condensed
          surpressAutoScroll
        />
      </div>
    </div>
  );
};

export default SearchPanel;
