import {
  CaretDoubleLeft,
  CaretDoubleRight,
  CaretLeft,
  CaretRight,
} from "@phosphor-icons/react";
import { CaretSortIcon } from "@radix-ui/react-icons";
import {
  ColumnDef,
  PaginationState,
  SortingState,
  VisibilityState,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
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
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { OrderDirection } from "@flow/types/paginationOptions";

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../Table";

type DataTableProps<TData, TValue> = {
  columns: ColumnDef<TData, TValue>[];
  data: TData[];
  selectColumns?: boolean;
  showFiltering?: boolean;
  enablePagination?: boolean;
  totalPages?: number;
  rowHeight?: number;
  onRowClick?: (row: TData) => void;
  currentPage?: number;
  setCurrentPage?: (page: number) => void;
  resultsPerPage?: number;
  currentOrder?: OrderDirection;
  setCurrentOrder?: (order: OrderDirection) => void;
};

function DataTable<TData, TValue>({
  columns,
  data,
  selectColumns = false,
  showFiltering = false,
  enablePagination = false,
  totalPages = 1,
  rowHeight,
  onRowClick,
  currentPage = 1,
  setCurrentPage,
  resultsPerPage,
  currentOrder = OrderDirection.Desc,
  setCurrentOrder,
}: DataTableProps<TData, TValue>) {
  const t = useT();
  const [sorting, setSorting] = useState<SortingState>([]);
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});
  const [rowSelection, setRowSelection] = useState({});
  const [globalFilter, setGlobalFilter] = useState("");
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: resultsPerPage ?? 10,
  });

  const defaultData = useMemo(() => [], []);
  const table = useReactTable({
    data: data ? data : defaultData,
    columns,
    getCoreRowModel: getCoreRowModel(),
    // Sorting
    onSortingChange: setSorting,
    getSortedRowModel: getSortedRowModel(),
    // Visibility
    onColumnVisibilityChange: setColumnVisibility,
    columnResizeMode: "onChange",
    // Row selection
    onRowSelectionChange: setRowSelection,
    // Filtering
    onGlobalFilterChange: setGlobalFilter,
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: enablePagination
      ? getPaginationRowModel()
      : undefined,
    onPaginationChange: setPagination,
    state: {
      sorting,
      columnVisibility,
      rowSelection,
      globalFilter,
      pagination,
    },
    manualPagination: true,
  });

  const handleOrderChange = () => {
    setCurrentOrder?.(
      currentOrder === OrderDirection.Asc
        ? OrderDirection.Desc
        : OrderDirection.Asc,
    );
  };

  return (
    <div className="flex flex-col justify-between">
      <div>
        <div className="flex items-center gap-4 py-4">
          {showFiltering && (
            <Input
              placeholder={t("Search") + "..."}
              value={globalFilter ?? ""}
              onChange={(e) => setGlobalFilter(String(e.target.value))}
              className="max-w-sm"
            />
          )}
          {currentOrder && (
            <IconButton
              size="icon"
              variant={"ghost"}
              tooltipText={t("By Ascending/Descending")}
              onClick={handleOrderChange}
              icon={<CaretSortIcon />}
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
                        {column.columnDef.header?.toString()}
                      </DropdownMenuCheckboxItem>
                    );
                  })}
              </DropdownMenuContent>
            </DropdownMenu>
          )}
        </div>

        <div className="rounded-md border">
          <Table>
            <TableHeader>
              {table.getHeaderGroups().map((headerGroup) => (
                <TableRow key={headerGroup.id}>
                  {headerGroup.headers.map((header) => {
                    return (
                      <TableHead key={header.id}>
                        {header.isPlaceholder
                          ? null
                          : flexRender(
                              header.column.columnDef.header,
                              header.getContext(),
                            )}
                      </TableHead>
                    );
                  })}
                </TableRow>
              ))}
            </TableHeader>
            <TableBody>
              {table.getRowModel().rows?.length ? (
                table.getRowModel().rows.map((row) => (
                  <TableRow
                    key={row.id}
                    className={`${rowHeight ? "h-" + rowHeight : "h-10"} cursor-pointer`}
                    data-state={row.getIsSelected() && "selected"}
                    onClick={() => {
                      row.toggleSelected();
                      onRowClick?.(row.original);
                    }}>
                    {row.getVisibleCells().map((cell) => (
                      <TableCell key={cell.id}>
                        {flexRender(
                          cell.column.columnDef.cell,
                          cell.getContext(),
                        )}
                      </TableCell>
                    ))}
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell
                    colSpan={columns.length}
                    className="h-24 text-center">
                    {t("No Results")}
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </div>
      </div>
      {enablePagination && (
        <div className="flex justify-center gap-4 pt-4">
          <div className="flex gap-1">
            <IconButton
              variant="outline"
              icon={<CaretDoubleLeft />}
              onClick={() => {
                if (currentPage > 1) {
                  setCurrentPage?.(1);
                  // table.setPageIndex(0);
                }
              }}
              disabled={currentPage <= 1}
            />
            <IconButton
              variant="outline"
              icon={<CaretLeft />}
              onClick={() => {
                if (currentPage > 1) {
                  setCurrentPage?.(currentPage - 1);
                  // table.previousPage();
                }
              }}
              disabled={currentPage <= 1}
            />
            <div className="flex min-w-10 items-center justify-center gap-1">
              <p className="text-sm font-light">{currentPage}</p>
              <p className="text-xs font-light">/</p>
              <p className="text-sm font-light">{totalPages}</p>
            </div>
            <IconButton
              className="rounded border p-1"
              icon={<CaretRight />}
              onClick={() => {
                if (currentPage < totalPages) {
                  setCurrentPage?.(currentPage + 1);
                  // table.nextPage();
                }
              }}
              disabled={currentPage >= totalPages}
            />

            <IconButton
              className="rounded border p-1"
              icon={<CaretDoubleRight />}
              onClick={() => {
                if (currentPage < totalPages) {
                  setCurrentPage?.(totalPages);
                  // table.setPageIndex(totalPages - 1);
                }
              }}
              disabled={currentPage >= totalPages}
            />
          </div>
        </div>
      )}
    </div>
  );
}

DataTable.displayName = "DataTable";

export { DataTable };
