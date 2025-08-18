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
import { useVirtualizer } from "@tanstack/react-virtual";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
  Button,
  Input,
  TableRow,
  TableCell,
  TableBody,
  TableHead,
  TableHeader,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type DataTableProps<TData, TValue> = {
  columns: ColumnDef<TData, TValue>[];
  data?: TData[];
  selectColumns?: boolean;
  showFiltering?: boolean;
  condensed?: boolean;
  searchTerm?: string;
  selectedRow?: any;
  useStrictSelectedRow?: boolean;
  onRowClick?: (row: TData) => void;
  onRowDoubleClick?: (row: TData) => void;
  setSearchTerm?: (term: string) => void;
};

function VirtualizedTable<TData, TValue>({
  columns,
  data,
  selectColumns = false,
  showFiltering = false,
  condensed,
  searchTerm,
  selectedRow,
  useStrictSelectedRow,
  onRowClick,
  onRowDoubleClick,
  setSearchTerm,
}: DataTableProps<TData, TValue>) {
  const t = useT();
  const [sorting, setSorting] = useState<SortingState>([]);
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});
  const [rowSelection, setRowSelection] = useState({});
  const [globalFilter, setGlobalFilter] = useState<string>("");

  useEffect(() => {
    if (searchTerm !== undefined) {
      setGlobalFilter(searchTerm);
    }
  }, [searchTerm]);

  const handleSearch = useCallback(
    (value: string) => {
      setGlobalFilter(value);
      setSearchTerm?.(value);
    },
    [setSearchTerm],
  );

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
    state: {
      sorting,
      columnVisibility,
      rowSelection,
      globalFilter,
    },
    manualPagination: true,
  });

  const parentRef = useRef<HTMLDivElement>(null);
  const { rows } = table.getRowModel();
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 24,
  });

  const selectedRowIndex = useMemo(() => {
    if (!selectedRow?.properties?._originalId || !data) return -1;
    return data.findIndex(
      (row: any) =>
        row.id?.replace(/[^a-zA-Z0-9]/g, "") ===
        selectedRow.id?.replace(/[^a-zA-Z0-9]/g, ""),
    );
  }, [selectedRow, data]);

  useEffect(() => {
    if (selectedRowIndex !== -1 && selectedRow.properties?._originalId) {
      virtualizer.scrollToIndex(selectedRowIndex, {
        align: "start",
        behavior: "auto",
      });
    }
  }, [selectedRowIndex, selectedRow, virtualizer]);

  return (
    <div className="flex h-full flex-col">
      {(showFiltering || selectColumns) && (
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

      <div
        ref={parentRef}
        className="h-full overflow-auto rounded-md border"
        style={{ contain: "paint", willChange: "transform" }}>
        <div
          className="w-full caption-bottom overflow-auto text-xs"
          style={{
            height: `${virtualizer.getTotalSize()}px`,
          }}>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
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
            {rows.length ? (
              virtualizer.getVirtualItems().map((virtualRow, idx) => {
                const row = rows[virtualRow.index] as any;
                const isSelected = selectedRowIndex === virtualRow.index;
                return (
                  <TableRow
                    key={row.id}
                    // Below is fix to ensure virtualized rows have a bottom border see: https://github.com/TanStack/virtual/issues/620
                    className="after:border-line-200 after:absolute after:top-0 after:left-0 after:z-10 after:w-full after:border-b relative cursor-pointer border-0"
                    style={{
                      height: `${virtualRow.size}px`,
                      transform: `translateY(${virtualRow.start - idx * virtualRow.size}px)`,
                    }}
                    data-state={
                      useStrictSelectedRow
                        ? selectedRow && isSelected
                          ? "selected"
                          : undefined
                        : row.getIsSelected()
                          ? "selected"
                          : undefined
                    }
                    onClick={() => {
                      row.toggleSelected();
                      onRowClick?.(row.original);
                    }}
                    onDoubleClick={() => {
                      onRowDoubleClick?.(row.original);
                    }}>
                    {row.getVisibleCells().map((cell: any) => {
                      return (
                        <TableCell
                          key={cell.id}
                          className={`${condensed ? "px-2 py-[2px]" : "p-2"}`}>
                          {flexRender(
                            cell.column.columnDef.cell,
                            cell.getContext(),
                          )}
                        </TableCell>
                      );
                    })}
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
        </div>
      </div>
    </div>
  );
}

export { VirtualizedTable };
