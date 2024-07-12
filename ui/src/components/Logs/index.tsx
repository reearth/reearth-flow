import {
  CaretSortIcon,
  ClockIcon,
  CrossCircledIcon,
  ExclamationTriangleIcon,
  InfoCircledIcon,
  UpdateIcon,
} from "@radix-ui/react-icons";
import {
  ColumnDef,
  SortingState,
  VisibilityState,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { useState } from "react";

import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
  Button,
  Input,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import { Table, TableBody, TableCell, TableRow } from "../Table";

interface LogProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[];
  data: TData[];
  selectColumns?: boolean;
  showFiltering?: boolean;
}

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
    state: {
      sorting,
      columnVisibility,
      rowSelection,
      globalFilter,
    },
  });

  return (
    <div>
      <div className="flex items-center py-4 gap-4">
        {showFiltering && (
          <Input
            placeholder={t("Search") + "..."}
            value={globalFilter ?? ""}
            onChange={e => setGlobalFilter(String(e.target.value))}
            className="max-w-sm"
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
                .filter(column => column.getCanHide())
                .map(column => {
                  return (
                    <DropdownMenuCheckboxItem
                      key={column.id}
                      className="capitalize"
                      checked={column.getIsVisible()}
                      onCheckedChange={value => column.toggleVisibility(!!value)}>
                      {column.id}
                    </DropdownMenuCheckboxItem>
                  );
                })}
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </div>
      <div className="text- bg-zinc-900 rounded text-white">
        <div className="border-b border-gray-400 h-16 flex w-full items-center justify-between p-2">
          <h2 className="text-lg">{t("Log")}</h2>
          <div className="flex gap-2">
            <Button variant="outline" size="icon">
              <CrossCircledIcon />
            </Button>
            <Button variant="outline" size="icon">
              <ExclamationTriangleIcon />
            </Button>
            <Button variant="outline" size="icon">
              <InfoCircledIcon />
            </Button>
            <Button variant="outline" size="icon">
              <ClockIcon />
            </Button>
            <Button variant="ghost" size="icon">
              <CaretSortIcon />
            </Button>
            <Button variant="ghost" size="icon">
              <UpdateIcon />
            </Button>
          </div>
        </div>
        <Table>
          <TableBody className="">
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map(row => (
                <TableRow key={row.id} data-state={row.getIsSelected() && "selected"}>
                  {row.getVisibleCells().map(cell => (
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
    </div>
  );
};

Logs.displayName = "Logs";

export { Logs };
