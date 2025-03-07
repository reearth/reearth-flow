import { Bug } from "@phosphor-icons/react";
import {
  CaretSortIcon,
  ClockIcon,
  CrossCircledIcon,
  ExclamationTriangleIcon,
  InfoCircledIcon,
  UpdateIcon,
  MagnifyingGlassIcon,
} from "@radix-ui/react-icons";
import {
  ColumnDef,
  ColumnFiltersState,
  SortingState,
  VisibilityState,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { useMemo, useState } from "react";

import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
  Button,
  Input,
  IconButton,
  FlowLogo,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Log, LogLevel } from "@flow/types";

import BasicBoiler from "../BasicBoiler";
import { Table, TableBody, TableCell, TableRow } from "../Table";

type LogProps = {
  columns: ColumnDef<Log, unknown>[];
  data: Log[];
  selectColumns?: boolean;
  showFiltering?: boolean;
};

const LogsTable = ({
  columns,
  data,
  selectColumns = false,
  showFiltering = false,
}: LogProps) => {
  const t = useT();
  const [sorting, setSorting] = useState<SortingState>([]);
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});
  const [rowSelection, setRowSelection] = useState({});
  const [globalFilter, setGlobalFilter] = useState("");
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([]);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    // Sorting
    onSortingChange: setSorting,
    getSortedRowModel: getSortedRowModel(),
    // Visibility
    onColumnVisibilityChange: setColumnVisibility,
    // Row selection
    onRowSelectionChange: setRowSelection,
    // Filtering
    onGlobalFilterChange: setGlobalFilter,
    getFilteredRowModel: getFilteredRowModel(),
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

  const handleStatusChange = (status: LogLevel) => {
    if (getStatusValue === status) {
      setColumnFilters([]);
    } else {
      setColumnFilters([{ id: "status", value: status }]);
    }
  };

  const handleTimeStampColumnVisibility = () => {
    const column = table.getColumn("timeStamp");

    column?.toggleVisibility(!column.getIsVisible());
    return;
  };

  const handleResetTable = () => {
    setColumnFilters([]);
    table.getColumn("timeStamp")?.toggleVisibility(true);
  };

  const getStatusValue = useMemo(() => {
    const value = columnFilters.find((id) => id.id === "status");
    return value?.value;
  }, [columnFilters]);

  const hasValidLogs = data.some(
    (log) => log.timeStamp || log.status || log.message,
  );

  return (
    <div className="flex size-full flex-col rounded">
      <div className="flex w-full items-center justify-between px-1 pb-2">
        <div className="mr-4 flex-1">
          {showFiltering && (
            <Input
              placeholder={t("Search") + "..."}
              value={globalFilter ?? ""}
              onChange={(e) => setGlobalFilter(String(e.target.value))}
              className="w-3/5 min-w-80"
            />
          )}
        </div>
        <div className="flex items-center gap-2">
          <IconButton
            size="icon"
            variant={getStatusValue === "ERROR" ? "default" : "outline"}
            tooltipText={t("Error")}
            onClick={() => handleStatusChange(LogLevel.ERROR)}
            icon={<CrossCircledIcon />}
          />
          <IconButton
            size="icon"
            variant={getStatusValue === "WARN" ? "default" : "outline"}
            tooltipText={t("Warning")}
            onClick={() => handleStatusChange(LogLevel.WARN)}
            icon={<ExclamationTriangleIcon />}
          />
          <IconButton
            size="icon"
            variant={getStatusValue === "DEBUG" ? "default" : "outline"}
            tooltipText={t("Debug")}
            onClick={() => handleStatusChange(LogLevel.DEBUG)}
            icon={<Bug />}
          />
          <IconButton
            size="icon"
            variant={getStatusValue === "TRACE" ? "default" : "outline"}
            tooltipText={t("Trace")}
            onClick={() => handleStatusChange(LogLevel.TRACE)}
            icon={<MagnifyingGlassIcon />}
          />
          <IconButton
            size="icon"
            variant={getStatusValue === "INFO" ? "default" : "outline"}
            tooltipText={t("Info")}
            onClick={() => handleStatusChange(LogLevel.INFO)}
            icon={<InfoCircledIcon />}
          />
          <IconButton
            size="icon"
            variant={
              table.getColumn("timeStamp")?.getIsVisible()
                ? "default"
                : "outline"
            }
            tooltipText={t("Include Time Stamp")}
            onClick={handleTimeStampColumnVisibility}
            icon={<ClockIcon />}
          />
          <Button variant="ghost" size="icon">
            <CaretSortIcon />
          </Button>
          <IconButton
            size="icon"
            variant="ghost"
            tooltipText={t("Reset Logs")}
            onClick={handleResetTable}
            icon={<UpdateIcon />}
          />
          {selectColumns && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="outline" size="sm" className="ml-auto">
                  {t("Columns")}
                </Button>
              </DropdownMenuTrigger>
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

      <div className="h-[calc(100vh-6rem)] w-full overflow-auto">
        <div className="border-b" />
        {!hasValidLogs || !table.getRowModel().rows?.length ? (
          <BasicBoiler
            className="h-full"
            textClassName="text-base"
            text={t("No Logs Available")}
            icon={<FlowLogo className="size-16 text-accent" />}
          />
        ) : (
          <Table>
            <TableBody>
              {table.getRowModel().rows.map((row) => (
                <TableRow
                  key={row.id}
                  className={`${row.original.status === "ERROR" ? "text-destructive" : row.original.status === "WARN" ? "text-warning" : ""}`}
                  data-state={row.getIsSelected() && "selected"}>
                  {row.getVisibleCells().map((cell) => (
                    <TableCell className="cursor-pointer" key={cell.id}>
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
