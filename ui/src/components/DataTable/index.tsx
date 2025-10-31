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
import { useCallback, useState } from "react";

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
  LoadingSkeleton,
  FlowLogo,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { OrderDirection } from "@flow/types/paginationOptions";

import BasicBoiler from "../BasicBoiler";
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
  data?: TData[];
  selectColumns?: boolean;
  showFiltering?: boolean;
  showOrdering?: boolean;
  enablePagination?: boolean;
  totalPages?: number;
  condensed?: boolean;
  currentPage?: number;
  resultsPerPage?: number;
  currentOrder?: OrderDirection;
  sortOptions?: { value: string; label: string }[];
  currentSortValue?: string;
  isFetching?: boolean;
  noResultsMessage?: string;
  onRowClick?: (row: TData) => void;
  onRowDoubleClick?: (row: TData) => void;
  onSortChange?: (value: string) => void;
  setCurrentPage?: (page: number) => void;
  setCurrentOrderDir?: (order: OrderDirection) => void;
  setSearchTerm?: (term: string) => void;
};

function DataTable<TData, TValue>({
  columns,
  data,
  selectColumns = false,
  showFiltering = false,
  showOrdering = true,
  enablePagination = false,
  totalPages = 1,
  condensed,
  currentPage = 1,
  resultsPerPage,
  currentOrder = OrderDirection.Desc,
  sortOptions,
  currentSortValue,
  isFetching,
  noResultsMessage,
  onRowClick,
  onRowDoubleClick,
  setCurrentPage,
  setCurrentOrderDir,
  onSortChange,
  setSearchTerm,
}: DataTableProps<TData, TValue>) {
  const t = useT();
  const [sorting, setSorting] = useState<SortingState>([]);
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});
  const [rowSelection, setRowSelection] = useState({});
  const [globalFilter, setGlobalFilter] = useState<string>("");

  const handleSearch = useCallback(
    (value: string) => {
      setGlobalFilter(value);
      setSearchTerm?.(value);
    },
    [setSearchTerm],
  );

  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: resultsPerPage ?? 10,
  });

  const table = useReactTable({
    data: data ?? [],
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
    setCurrentOrderDir?.(
      currentOrder === OrderDirection.Asc
        ? OrderDirection.Desc
        : OrderDirection.Asc,
    );
  };

  const orderDirections: Record<OrderDirection, string> = {
    DESC: t("Newest"),
    ASC: t("Oldest"),
  };

  const { rows } = table.getRowModel();

  return (
    <div className="flex h-full flex-col justify-between">
      {(showOrdering || showFiltering || selectColumns) && (
        <div
          className={`flex items-center gap-4 ${condensed ? "py-1" : "py-3"}`}>
          {showFiltering && (
            <Input
              placeholder={t("Search") + "..."}
              value={globalFilter}
              onChange={(e) => {
                const value = String(e.target.value);
                handleSearch(value);
              }}
              className="h-[36px] max-w-[220px]"
            />
          )}
          {showOrdering && sortOptions && onSortChange ? (
            <Select value={currentSortValue} onValueChange={onSortChange}>
              <SelectTrigger className="h-[32px] w-[150px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {sortOptions.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          ) : showOrdering ? (
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
          ) : null}

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
      )}
      {isFetching ? (
        <LoadingSkeleton />
      ) : rows.length ? (
        <div className="flex-1 overflow-auto">
          <div
            className="overflow-auto rounded-md border"
            style={{ contain: "paint", willChange: "transform" }}>
            <Table>
              <TableHeader className="sticky top-0 z-10 bg-background/50 backdrop-blur-2xl">
                {table.getHeaderGroups().map((headerGroup) => (
                  <TableRow
                    key={headerGroup.id}
                    className="bg-background/50 backdrop-blur-2xl">
                    {headerGroup.headers.map((header) => {
                      return (
                        <TableHead
                          key={header.id}
                          className={`${condensed ? "h-8" : "h-10"}`}>
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
                {rows.length &&
                  rows.map((row) => {
                    return (
                      <TableRow
                        key={row.id}
                        data-state={
                          row.getIsSelected() ? "selected" : undefined
                        }
                        onClick={() => {
                          row.toggleSelected();
                          onRowClick?.(row.original);
                        }}
                        onDoubleClick={() => {
                          onRowDoubleClick?.(row.original);
                        }}>
                        {row.getVisibleCells().map((cell: any) => (
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
                  })}
              </TableBody>
            </Table>
          </div>
        </div>
      ) : (
        <BasicBoiler
          text={noResultsMessage || t("No Results")}
          icon={<FlowLogo className="size-16 text-accent" />}
        />
      )}
      {enablePagination && rows.length > 0 && (
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
