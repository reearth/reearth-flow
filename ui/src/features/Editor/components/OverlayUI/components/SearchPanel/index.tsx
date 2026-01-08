import { XIcon } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";
import { useState } from "react";

import { IconButton } from "@flow/components";
import { VirtualizedTable } from "@flow/components/visualizations/VirtualizedTable";
import { SearchNodeResult, useSearchNodes } from "@flow/hooks/useSearchNodes";
import { useT } from "@flow/lib/i18n";
import { Workflow } from "@flow/types";

type SearchPanelProps = {
  rawWorkflows: Workflow[];
  currentWorkflowId: string;
  onWorkflowOpen: (id: string) => void;
  onPopoverClose?: () => void;
};

export const SearchPanel = ({
  rawWorkflows,
  currentWorkflowId,
  onWorkflowOpen,
  onPopoverClose,
}: SearchPanelProps) => {
  const t = useT();
  const { allNodes, handleNavigateToNode } = useSearchNodes({
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
        <span className="font-medium">{row.original.displayName}</span>
      ),
    },
    {
      accessorKey: "workflowName",
      header: "Workflow",
      cell: ({ row }) => (
        <span className="text-muted-foreground">
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
      <div className="flex items-center justify-between">
        <span className="text-center">{t("Search Actions")}</span>
        <IconButton
          variant="ghost"
          onClick={onPopoverClose}
          icon={<XIcon size={16} />}
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
