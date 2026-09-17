import { ColumnDef } from "@tanstack/react-table";
import { useMemo } from "react";

import useDiagnosticLabels from "@flow/hooks/useDiagnosticLabels";
import { useT } from "@flow/lib/i18n";
import {
  type Diagnostic,
  diagnosticOccurrences,
  isAggregatedDiagnostic,
} from "@flow/types";

import { Badge } from "../Badge";
import { DataTable as Table } from "../DataTable";

type Props = {
  diagnostics: Diagnostic[];
  isFetching?: boolean;
  noResultsMessage?: string;
  /** Controls rendered beside the table's search input. */
  leadingActions?: React.ReactNode;
  /** Edge-to-edge console styling, to match LogsTable when swapped with it. */
  flush?: boolean;
  /** Double-clicking a row, e.g. to reveal the action that reported it. */
  onRowDoubleClick?: (diagnostic: Diagnostic) => void;
};

// Severity is a display level only, so it drives nothing but the colour here.
const severityBadgeClasses: Record<string, string> = {
  fatal: "bg-destructive text-destructive-foreground",
  error: "bg-destructive text-destructive-foreground",
  warn: "bg-warning text-warning-foreground",
};

const DiagnosticsTable: React.FC<Props> = ({
  diagnostics,
  isFetching,
  noResultsMessage,
  leadingActions,
  flush,
  onRowDoubleClick,
}) => {
  const t = useT();
  const { severityLabel, categoryLabel } = useDiagnosticLabels();

  const columns: ColumnDef<Diagnostic>[] = useMemo(
    () => [
      {
        accessorKey: "severity",
        header: t("Severity"),
        cell: ({ row }) => {
          const severity = row.original.severity;
          return (
            <Badge
              variant="secondary"
              className={severityBadgeClasses[severity] ?? ""}>
              {severityLabel(severity)}
            </Badge>
          );
        },
      },

      {
        accessorKey: "category",
        header: t("Category"),
        cell: ({ row }) => categoryLabel(row.original.category),
      },
      {
        accessorKey: "actionType",
        header: t("Action Type"),
        cell: ({ row }) => row.original.actionType ?? t("N/A"),
      },
      {
        id: "occurrences",
        accessorFn: (diagnostic) => diagnosticOccurrences(diagnostic),
        header: t("Occurrences"),
        cell: ({ row }) =>
          // An aggregated row stands for many features; every other row is a
          // single occurrence. Never parse the count out of the message.
          isAggregatedDiagnostic(row.original)
            ? diagnosticOccurrences(row.original).toLocaleString()
            : "1",
      },
      {
        accessorKey: "message",
        header: t("Message"),
        cell: ({ row }) => {
          const { message, help } = row.original;
          return (
            <div className="flex flex-col gap-1">
              <p>{message}</p>
              {help && (
                <p className="text-xs font-light text-muted-foreground">
                  {help}
                </p>
              )}
            </div>
          );
        },
      },
      {
        accessorKey: "code",
        header: t("Code"),
      },
      {
        id: "features",
        accessorFn: (diagnostic) =>
          diagnostic.featureId ?? diagnostic.sampleFeatureIds?.join(" ") ?? "",
        header: t("Features"),
        cell: ({ row }) => {
          const { featureId, sampleFeatureIds } = row.original;
          if (featureId) return featureId;
          if (!sampleFeatureIds?.length) return t("N/A");
          return (
            <div className="flex flex-col gap-1">
              <p className="text-xs font-light text-muted-foreground">
                {t("Samples: {{amount}}", { amount: sampleFeatureIds.length })}
              </p>
              {sampleFeatureIds.map((id) => (
                <p key={id} className="text-xs font-light">
                  {id}
                </p>
              ))}
            </div>
          );
        },
      },
    ],
    [t, severityLabel, categoryLabel],
  );

  return (
    <Table
      columns={columns}
      data={diagnostics}
      condensed
      selectColumns
      showFiltering
      showOrdering={false}
      isFetching={isFetching}
      leadingActions={leadingActions}
      flush={flush}
      onRowDoubleClick={onRowDoubleClick}
      noResultsMessage={noResultsMessage ?? t("No diagnostics")}
    />
  );
};

export { DiagnosticsTable };
