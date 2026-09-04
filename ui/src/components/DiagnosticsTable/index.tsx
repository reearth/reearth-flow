import { ColumnDef } from "@tanstack/react-table";
import { useMemo } from "react";

import useDiagnosticLabels from "@flow/hooks/useDiagnosticLabels";
import { useT } from "@flow/lib/i18n";
import {
  type Diagnostic,
  diagnosticOccurrences,
  isAggregatedDiagnostic,
  isFatalDiagnostic,
} from "@flow/types";

import { Badge } from "../Badge";
import { DataTable as Table } from "../DataTable";

type Props = {
  diagnostics: Diagnostic[];
  isFetching?: boolean;
  noResultsMessage?: string;
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
}) => {
  const t = useT();
  const { severityLabel, dispositionLabel, categoryLabel } =
    useDiagnosticLabels();

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
        accessorKey: "effectiveDisposition",
        header: t("Disposition"),
        cell: ({ row }) => {
          // The only field that says whether the run actually failed, so it is
          // spelled out rather than folded into the severity badge.
          return (
            <span
              className={
                isFatalDiagnostic(row.original)
                  ? "text-destructive"
                  : "font-light"
              }>
              {dispositionLabel(row.original.effectiveDisposition) ?? t("N/A")}
            </span>
          );
        },
      },
      {
        accessorKey: "code",
        header: t("Code"),
      },
      {
        accessorKey: "category",
        header: t("Category"),
        cell: ({ row }) => categoryLabel(row.original.category),
      },
      {
        accessorKey: "nodeId",
        header: t("Action Id"),
        cell: ({ row }) => row.original.nodeId ?? t("N/A"),
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
    [t, severityLabel, dispositionLabel, categoryLabel],
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
      noResultsMessage={noResultsMessage ?? t("No diagnostics")}
    />
  );
};

export { DiagnosticsTable };
