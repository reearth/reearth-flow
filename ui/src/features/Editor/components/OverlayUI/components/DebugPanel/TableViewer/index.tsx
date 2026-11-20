import { CircleNotchIcon } from "@phosphor-icons/react";
import { memo } from "react";

import BasicBoiler from "@flow/components/BasicBoiler";
import type { FeatureRows, FeatureTableRow } from "@flow/lib/featureRows";
import { useLang, useT } from "@flow/lib/i18n";

import FeatureRowsTable from "./FeatureRowsTable";

type Props = {
  rows: FeatureRows;
  selectedRow: number | null;
  onSingleClick?: (row: FeatureTableRow) => void;
  onDoubleClick?: (row: FeatureTableRow) => void;
};

const TableViewer: React.FC<Props> = memo(
  ({ rows, selectedRow, onSingleClick, onDoubleClick }) => {
    const t = useT();
    // Counts are grouped for the UI language, not the browser's: `1.234` and
    // `1,234` are different numbers depending on where you read them.
    const lang = useLang();
    const { status } = rows;

    if (status?.error && status.rowCount === 0) {
      return <BasicBoiler text={status.error} className="h-full" />;
    }
    if (!status || (!status.done && status.rowCount === 0)) {
      return <BasicBoiler text={t("Loading data...")} className="h-full" />;
    }
    if (status.rowCount === 0) {
      return (
        <BasicBoiler
          text={t("No data to display in table format")}
          className="h-full"
        />
      );
    }

    const busy = !status.done || status.searching;

    return (
      <div className="relative flex h-full flex-col">
        <div className="flex h-full flex-1 flex-col">
          {status.error && (
            <div
              role="alert"
              className="mb-1 rounded-md border border-destructive/50 px-2 py-1 text-xs text-destructive">
              {status.error}
            </div>
          )}
          <div className="flex-1 overflow-hidden">
            <FeatureRowsTable
              rows={rows}
              selectedRow={selectedRow}
              onRowClick={onSingleClick}
              onRowDoubleClick={onDoubleClick}
            />
          </div>

          <div className="mt-1 flex items-center justify-between rounded-md px-2 text-xs text-muted-foreground">
            <div className="flex items-center gap-4">
              <span className="flex items-center gap-1">
                {t("Rows")}: {status.viewCount.toLocaleString(lang)}
                {` / ${status.totalRows.toLocaleString(lang)} ${t("total")}`}
                {busy && <CircleNotchIcon className="size-3 animate-spin" />}
              </span>
              {status.geometryType && (
                <span className="rounded px-2 text-xs">
                  {status.geometryType}
                </span>
              )}
              <span>
                {t("Columns")}:{" "}
                {(status.columns.length + 1).toLocaleString(lang)}
              </span>
              {status.truncated && (
                <span className="text-warning">
                  {t("Showing the first {{rows}} rows (memory limit reached)", {
                    rows: status.rowCount.toLocaleString(lang),
                  })}
                </span>
              )}
              {status.skippedLines > 0 && (
                <span className="text-warning">
                  {t("{{lines}} unreadable line(s) skipped", {
                    lines: status.skippedLines.toLocaleString(lang),
                  })}
                </span>
              )}
            </div>
          </div>
        </div>
      </div>
    );
  },
);

TableViewer.displayName = "TableViewer";

export default TableViewer;
