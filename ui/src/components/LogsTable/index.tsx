import {
  CaretSortIcon,
  ClockIcon,
  CrossCircledIcon,
  InfoCircledIcon,
  UpdateIcon,
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
  LoadingSkeleton,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { UserFacingLog, UserFacingLogLevel } from "@flow/types";

import BasicBoiler from "../BasicBoiler";
import { Table, TableBody, TableCell, TableRow } from "../Table";

type LogProps = {
  columns: ColumnDef<UserFacingLog, unknown>[];
  data: UserFacingLog[];
  isFetching: boolean;
  selectColumns?: boolean;
  showFiltering?: boolean;
};

const LogsTable = ({
  columns,
  data,
  isFetching,
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

  const handleStatusChange = (status: UserFacingLogLevel) => {
    if (getStatusValue === status) {
      setColumnFilters([]);
    } else {
      setColumnFilters([{ id: "level", value: status }]);
    }
  };

  const handleTimeStampColumnVisibility = () => {
    const column = table.getColumn("timestamp");

    column?.toggleVisibility(!column.getIsVisible());
    return;
  };

  const handleResetTable = () => {
    setColumnFilters([]);
    table.getColumn("timestamp")?.toggleVisibility(true);
  };

  const getStatusValue = useMemo(() => {
    const value = columnFilters.find((id) => id.id === "level");
    return value?.value;
  }, [columnFilters]);

  const hasValidLogs = data.some(
    (log) => log.timestamp || log.level || log.message,
  );

  return (
    <div className="flex size-full flex-col rounded">
      <div className="flex w-full shrink-0 items-center justify-between px-2 pb-2">
        <div className="mr-4 flex-1">
          {showFiltering && (
            <Input
              placeholder={t("Search") + "..."}
              value={globalFilter ?? ""}
              onChange={(e) => setGlobalFilter(String(e.target.value))}
            />
          )}
        </div>
        <div className="flex items-center gap-2">
          <IconButton
            size="icon"
            variant={getStatusValue === "ERROR" ? "default" : "outline"}
            tooltipText={t("Error")}
            onClick={() => handleStatusChange(UserFacingLogLevel.Error)}
            icon={<CrossCircledIcon />}
          />
          <IconButton
            size="icon"
            variant={getStatusValue === "INFO" ? "default" : "outline"}
            tooltipText={t("Info")}
            onClick={() => handleStatusChange(UserFacingLogLevel.Info)}
            icon={<InfoCircledIcon />}
          />
          <IconButton
            size="icon"
            variant={
              table.getColumn("timestamp")?.getIsVisible()
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
