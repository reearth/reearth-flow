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
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { LogLevel } from "@flow/types";

import { Table, TableBody, TableCell, TableRow } from "../Table";

type LogProps<TData, TValue> = {
  columns: ColumnDef<TData, TValue>[];
  data: TData[];
  selectColumns?: boolean;
  showFiltering?: boolean;
};

const Logs = <TData, TValue>({
  columns,
  data,
  selectColumns = false,
  showFiltering = false,
}: LogProps<TData, TValue>) => {
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
      setColumnFilters([{ id: "logLevel", value: status }]);
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
    const value = columnFilters.find((id) => id.id === "logLevel");
    return value?.value;
  }, [columnFilters]);
  console.log("data", data);
  return (
    <div className="w-full overflow-auto rounded">
      <div className="flex h-16 w-full items-center justify-between p-2">
        <h2 className="text-lg">{t("Log")}</h2>
        <div className="flex gap-2">
          <Button
            variant={getStatusValue === "ERROR" ? "default" : "outline"}
            size="icon"
            onClick={() => handleStatusChange(LogLevel.ERROR)}>
            <CrossCircledIcon />
          </Button>
          <Button
            variant={getStatusValue === "WARN" ? "default" : "outline"}
            size="icon"
            onClick={() => handleStatusChange(LogLevel.WARN)}>
            <ExclamationTriangleIcon />
          </Button>
          <Button
            variant={getStatusValue === "DEBUG" ? "default" : "outline"}
            size="icon"
            onClick={() => handleStatusChange(LogLevel.DEBUG)}>
            <Bug />
          </Button>
          <Button
            variant={getStatusValue === "TRACE" ? "default" : "outline"}
            size="icon"
            onClick={() => handleStatusChange(LogLevel.TRACE)}>
            <MagnifyingGlassIcon />
          </Button>
          <Button
            variant={getStatusValue === "INFO" ? "default" : "outline"}
            size="icon"
            onClick={() => handleStatusChange(LogLevel.INFO)}>
            <InfoCircledIcon />
          </Button>
          <Button
            variant={
              table.getColumn("timestamp")?.getIsVisible()
                ? "default"
                : "outline"
            }
            size="icon"
            onClick={handleTimeStampColumnVisibility}>
            <ClockIcon />
          </Button>
          <Button variant="ghost" size="icon">
            <CaretSortIcon />
          </Button>
          <Button variant="ghost" size="icon" onClick={handleResetTable}>
            <UpdateIcon />
          </Button>
        </div>
      </div>
      <div className="flex items-center gap-4 p-4">
        {showFiltering && (
          <Input
            placeholder={t("Search") + "..."}
            value={globalFilter ?? ""}
            onChange={(e) => setGlobalFilter(String(e.target.value))}
            className="max-w-80"
          />
        )}
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
                      {column.id}
                    </DropdownMenuCheckboxItem>
                  );
                })}
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </div>
      <div className="border-b border-gray-400" />
      <Table>
        <TableBody>
          {table.getRowModel().rows?.length ? (
            table.getRowModel().rows.map((row) => (
              <TableRow
                key={row.id}
                data-state={row.getIsSelected() && "selected"}>
                {row.getVisibleCells().map((cell) => (
                  <TableCell className="cursor-pointer" key={cell.id}>
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </TableCell>
                ))}
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={columns.length} className="h-24 text-center">
                {t("No Results")}
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
};

Logs.displayName = "Logs";

export { Logs };
