import {
  ArrowsClockwiseIcon,
  CheckCircleIcon,
  InfoIcon,
  XCircleIcon,
} from "@phosphor-icons/react";
import type {
  ColumnFiltersState,
  ColumnVisibilityState,
  SortingState,
} from "@tanstack/react-table";
import { flexRender, useTable } from "@tanstack/react-table";
import { useState } from "react";

import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
  Button,
  Input,
  IconButton,
  FlowLogo,
  LoadingSkeleton,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { appTableFeatures } from "@flow/lib/table/features";
import type { AppColumnDef } from "@flow/lib/table/features";
import type { UserFacingLog } from "@flow/types";
import { UserFacingLogLevel } from "@flow/types";

import BasicBoiler from "../BasicBoiler";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../Table";

type LogProps = {
  columns: AppColumnDef<UserFacingLog, unknown>[];
  data: UserFacingLog[];
  isFetching: boolean;
  selectColumns?: boolean;
  showFiltering?: boolean;
  /** Controls rendered beside the search input, e.g. a view switch. */
  leadingActions?: React.ReactNode;
};

const LogsTable = ({
  columns,
  data,
  isFetching,
  selectColumns = false,
  showFiltering = false,
  leadingActions,
}: LogProps) => {
  const t = useT();
  const [sorting, setSorting] = useState<SortingState>([]);
  const [columnVisibility, setColumnVisibility] =
    useState<ColumnVisibilityState>({});
  const [rowSelection, setRowSelection] = useState({});
  const [globalFilter, setGlobalFilter] = useState("");
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([]);

  const table = useTable({
    features: appTableFeatures,
    data,
    columns: columns as AppColumnDef<UserFacingLog>[],
    // Sorting
    onSortingChange: setSorting,
    // Visibility
    onColumnVisibilityChange: setColumnVisibility,
    // Row selection
    onRowSelectionChange: setRowSelection,
    // Filtering
    onGlobalFilterChange: setGlobalFilter,
    // Column Filtering
    onColumnFiltersChange: setColumnFilters,
    state: {
      sorting,
      columnVisibility,
      rowSelection,
      globalFilter,
      columnFilters,
    },
  });

  const handleStatusChange = (status: UserFacingLogLevel) => {
    if (statusValue === status) {
      setColumnFilters([]);
    } else {
      setColumnFilters([{ id: "level", value: status }]);
    }
  };

  const handleResetTable = () => {
    setColumnFilters([]);
  };

  // Plain computation, not a useMemo: react-table v8 made React Compiler skip
  // this component wholesale ("Use of incompatible library"), which hid the
  // fact that it could not preserve this memo. v9 compiles fine, so the memo
  // now surfaces as an error — and a `.find` over the active filters is not
  // worth memoizing by hand anyway.
  const statusValue = columnFilters.find((id) => id.id === "level")?.value;

  const hasValidLogs = data.some(
    (log) => log.timestamp || log.level || log.message,
  );

  return (
    <div className="flex size-full flex-col rounded">
      <div className="flex w-full shrink-0 items-center justify-between px-2 pb-2">
        <div className="mr-4 flex items-center gap-2">
          {showFiltering && (
            <Input
              className="h-[36px] w-[25vw]"
              placeholder={t("Search") + "..."}
              value={globalFilter ?? ""}
              onChange={(e) => setGlobalFilter(String(e.target.value))}
            />
          )}
          {leadingActions}
        </div>
        <div className="flex items-center gap-2">
          <IconButton
            size="icon"
            variant={statusValue === "ERROR" ? "default" : "outline"}
            tooltipText={t("Error")}
            onClick={() => handleStatusChange(UserFacingLogLevel.Error)}
            icon={<XCircleIcon className="text-destructive" />}
          />
          <IconButton
            size="icon"
            variant={statusValue === "INFO" ? "default" : "outline"}
            tooltipText={t("Info")}
            onClick={() => handleStatusChange(UserFacingLogLevel.Info)}
            icon={<InfoIcon />}
          />
          <IconButton
            size="icon"
            variant={statusValue === "SUCCESS" ? "default" : "outline"}
            tooltipText={t("Success")}
            onClick={() => handleStatusChange(UserFacingLogLevel.Success)}
            icon={<CheckCircleIcon className="text-success" />}
          />
          <IconButton
            size="icon"
            variant="ghost"
            tooltipText={t("Reset Logs")}
            onClick={handleResetTable}
            icon={<ArrowsClockwiseIcon />}
          />
          {selectColumns && (
            <DropdownMenu>
              <DropdownMenuTrigger
                render={
                  <Button variant="outline" size="sm" className="ml-auto">
                    {t("Columns")}
                  </Button>
                }
              />
              <DropdownMenuContent align="end">
                {table
                  .getAllColumns()
                  .filter((column) => column.getCanHide())
                  .map((column) => {
                    return (
                      <DropdownMenuCheckboxItem
                        key={column.id}
                        className="capitalize"
                        checked={column.getIsVisible()}
                        onCheckedChange={(value) =>
                          column.toggleVisibility(!!value)
                        }>
                        {column.columnDef.header?.toString()}
                      </DropdownMenuCheckboxItem>
                    );
                  })}
              </DropdownMenuContent>
            </DropdownMenu>
          )}
        </div>
      </div>

      <div className="border-b" />
      <div className="h-[calc(100%-20px)] w-full overflow-auto">
        {isFetching ? (
          <LoadingSkeleton />
        ) : !hasValidLogs || !table.getRowModel().rows?.length ? (
          <BasicBoiler
            className="h-full"
            textClassName="text-base"
            text={t("No Logs Available")}
            icon={<FlowLogo className="size-16 text-accent" />}
          />
        ) : (
          <Table>
            {/* Matches DataTable's header so switching between the log and
                diagnostics views does not reflow the panel. */}
            <TableHeader className="sticky top-0 z-10 bg-card/50 backdrop-blur-2xl dark:bg-background/50">
              {table.getHeaderGroups().map((headerGroup) => (
                <TableRow
                  key={headerGroup.id}
                  className="bg-card/50 backdrop-blur-2xl dark:bg-background/50">
                  {headerGroup.headers.map((header) => (
                    <TableHead key={header.id} className="h-8">
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext(),
                          )}
                    </TableHead>
                  ))}
                </TableRow>
              ))}
            </TableHeader>
            <TableBody>
              {table.getRowModel().rows.map((row) => (
                <TableRow
                  key={row.id}
                  className={` ${row.original.level === "ERROR" ? "text-destructive" : row.original.level === "SUCCESS" ? "text-success/80" : ""}`}
                  data-state={row.getIsSelected() && "selected"}>
                  {row.getVisibleCells().map((cell) => (
                    <TableCell
                      className="cursor-pointer overflow-scroll"
                      key={cell.id}>
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext(),
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </div>
    </div>
  );
};

LogsTable.displayName = "LogsTable";

export { LogsTable };
