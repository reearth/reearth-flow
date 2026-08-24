import { ColumnDef } from "@tanstack/react-table";
import { useMemo } from "react";

import useDiagnosticLabels from "@flow/hooks/useDiagnosticLabels";
import { useT } from "@flow/lib/i18n";
import { type NodeExecution } from "@flow/types";
import { formatTimestamp } from "@flow/utils";

import { Badge } from "../Badge";
import { DataTable as Table } from "../DataTable";

type Props = {
  nodeExecutions: NodeExecution[];
  isFetching?: boolean;
  noResultsMessage?: string;
};

const statusBadgeClasses: Record<string, string> = {
  completed: "bg-success text-background",
  failed: "bg-destructive text-destructive-foreground",
};

const NodeExecutionsTable: React.FC<Props> = ({
  nodeExecutions,
  isFetching,
  noResultsMessage,
}) => {
  const t = useT();
  const { statusLabel } = useDiagnosticLabels();

  const columns: ColumnDef<NodeExecution>[] = useMemo(
    () => [
      {
        accessorKey: "nodeId",
        header: t("Action Id"),
      },
      {
        accessorKey: "status",
        header: t("Status"),
        cell: ({ row }) => {
          const status = row.original.status;
          return (
            <Badge
              variant="secondary"
              className={statusBadgeClasses[status] ?? ""}>
              {statusLabel(status)}
            </Badge>
          );
        },
      },
      // The three counts get their own columns on purpose: exactly one of them
      // is populated per node kind, but an empty cell means "not applicable, or
      // not finished yet" — never zero — so collapsing them into one column
      // would read as an inference about the node's kind that isn't safe to make.
      {
        accessorKey: "featuresProcessed",
        header: t("Features Processed"),
        cell: ({ row }) =>
          row.original.featuresProcessed?.toLocaleString() ?? t("N/A"),
      },
      {
        accessorKey: "featuresWritten",
        header: t("Features Written"),
        cell: ({ row }) =>
          row.original.featuresWritten?.toLocaleString() ?? t("N/A"),
      },
      {
        accessorKey: "finishFeatureCount",
        header: t("Finish Features"),
        cell: ({ row }) =>
          row.original.finishFeatureCount?.toLocaleString() ?? t("N/A"),
      },
      {
        id: "diagnostics",
        accessorFn: (nodeExecution) => nodeExecution.diagnostics?.length ?? 0,
        header: t("Diagnostics"),
      },
      {
        accessorKey: "startedAt",
        header: t("Started At"),
        cell: ({ row }) =>
          row.original.startedAt
            ? formatTimestamp(row.original.startedAt)
            : t("N/A"),
      },
      {
        accessorKey: "completedAt",
        header: t("Completed At"),
        cell: ({ row }) =>
          row.original.completedAt
            ? formatTimestamp(row.original.completedAt)
            : t("N/A"),
      },
    ],
    [t, statusLabel],
  );

  return (
    <Table
      columns={columns}
      data={nodeExecutions}
      condensed
      selectColumns
      showFiltering
      showOrdering={false}
      isFetching={isFetching}
      noResultsMessage={noResultsMessage ?? t("No action executions")}
    />
  );
};

export { NodeExecutionsTable };
