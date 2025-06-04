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
import { useVirtualizer } from "@tanstack/react-virtual";
import { useMemo, useRef, useState } from "react";

import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
  Button,
  Input,
  Pagination,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
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
  condensed?: boolean;
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
  condensed,
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

  const orderDirections: Record<OrderDirection, string> = {
    DESC: t("Newest"),
    ASC: t("Oldest"),
  };
  const parentRef = useRef<HTMLDivElement>(null);
  const { rows } = table.getRowModel();
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 34,
  });

  console.log(
    "rows",
    rows.length,
    "virtualizer",
    virtualizer.getVirtualItems().length,
  );

  return (
    <div className="flex h-full flex-col justify-between">
      <div className="flex h-full flex-col">
        <div
          className={`flex items-center gap-4 ${condensed ? "py-1" : "py-3"}`}>
          {showFiltering && (
            <Input
              placeholder={t("Search") + "..."}
              value={globalFilter ?? ""}
              onChange={(e) => setGlobalFilter(String(e.target.value))}
              className="max-w-sm"
            />
          )}
          {currentOrder && (
            <Select
              value={currentOrder || "DESC"}
              onValueChange={handleOrderChange}>
              <SelectTrigger className="h-[32px] w-[100px]">
                <SelectValue placeholder={orderDirections.ASC} />
              </SelectTrigger>
              <SelectContent>
                {Object.entries(orderDirections).map(([value, label]) => (
                  <SelectItem key={value} value={value}>
                    {label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
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
          <div
            ref={parentRef}
            style={{ height: 500, overflow: "auto" }}
            className="rounded-md border">
            <Table>
              <TableHeader>
                {table.getHeaderGroups().map((headerGroup) => (
                  <TableRow key={headerGroup.id}>
                    {headerGroup.headers.map((header) => {
                      return (
                        <TableHead
                          key={header.id}
                          className={`${condensed ? "h-8" : "h-10"} whitespace-nowrap`}>
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
                {rows.length ? (
                  virtualizer.getVirtualItems().map((virtualRow, idx) => {
                    const row = rows[virtualRow.index];
                    return (
                      <TableRow
                        key={row.id}
                        className="cursor-pointer"
                        style={{
                          height: `${virtualRow.size}px`,
                          transform: `translateY(${virtualRow.start - idx * virtualRow.size}px)`,
                        }}
                        data-state={row.getIsSelected() && "selected"}
                        onClick={() => {
                          row.toggleSelected();
                          onRowClick?.(row.original);
                        }}>
                        {row.getVisibleCells().map((cell) => (
                          <TableCell
                            key={cell.id}
                            className={`${condensed ? "px-2 py-[2px]" : "p-2"}`}>
                            {flexRender(
                              cell.column.columnDef.cell,
                              cell.getContext(),
                            )}
                          </TableCell>
                        ))}
                      </TableRow>
                    );
                  })
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
      </div>

      {enablePagination && (
        <Pagination
          currentPage={currentPage}
          setCurrentPage={setCurrentPage}
          totalPages={totalPages}
        />
      )}
    </div>
  );
}

DataTable.displayName = "DataTable";

export { DataTable };
