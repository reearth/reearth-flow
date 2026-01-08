import { XIcon } from "@phosphor-icons/react";
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
  onShowSearchPanel: (boolean: boolean) => void;
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
      header: "Action Name",
      cell: ({ row }) => (
        <span className="block max-w-[100px] truncate font-medium">
          {row.original.displayName}
        </span>
      ),
    },
    {
      accessorKey: "workflowName",
      header: "Workflow",
      cell: ({ row }) => (
        <span className="block max-w-[100px] truncate font-medium text-muted-foreground">
          {row.original.workflowName}
        </span>
      ),
    },
    {
      accessorKey: "nodeType",
      header: "Type",
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
    <div className="flex h-full flex-col gap-2 p-2">
      <div className="relative flex items-center justify-between">
        <span className="text-center">{t("Search Actions")}</span>
        <XIcon
          className="absolute top-1 right-1 cursor-pointer"
          onClick={() => onShowSearchPanel(false)}
        />
      </div>
      <div className="flex h-full flex-col">
        <VirtualizedTable
          columns={searchNodeColumns}
          data={allNodes}
          showFiltering={true}
          selectedFeatureId={selectedNodeId}
          onRowClick={handleRowClick}
          onRowDoubleClick={handleRowDoubleClick}
        />
      </div>
    </div>
  );
};

export default SearchPanel;
