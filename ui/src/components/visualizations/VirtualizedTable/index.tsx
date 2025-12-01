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
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  Button,
  Input,
  TableRow,
  TableCell,
  TableBody,
  TableHead,
  TableHeader,
  Table,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type DataTableProps<TData, TValue> = {
  columns: ColumnDef<TData, TValue>[];
  data?: TData[];
  selectColumns?: boolean;
  showFiltering?: boolean;
  condensed?: boolean;
  searchTerm?: string;
  selectedFeatureId?: string | null;
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
  selectedFeatureId,
  onRowClick,
  onRowDoubleClick,
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

  // Handle select all columns
  const handleSelectAllColumns = useCallback(() => {
    const visibilityUpdate: Record<string, boolean> = {};
    table
      .getAllColumns()
      .filter((column) => column.getCanHide())
      .forEach((column) => {
        visibilityUpdate[column.id] = true;
      });
    setColumnVisibility(visibilityUpdate);
  }, [table]);

  // Handle deselect all columns
  const handleDeselectAllColumns = useCallback(() => {
    const visibilityUpdate: Record<string, boolean> = {};
    table
      .getAllColumns()
      .filter((column) => column.getCanHide())
      .forEach((column) => {
        visibilityUpdate[column.id] = false;
      });
    setColumnVisibility(visibilityUpdate);
  }, [table]);

  const parentRef = useRef<HTMLDivElement>(null);
  const { rows } = table.getRowModel();
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 24,
  });

  const selectedRowIndex = useMemo(() => {
    if (!selectedFeatureId || !data) return -1;
    const normalizedSelectedId = String(selectedFeatureId).replace(
      /[^a-zA-Z0-9]/g,
      "",
    );
    return data.findIndex(
      (row: any) =>
        String(row.id || "").replace(/[^a-zA-Z0-9]/g, "") ===
        normalizedSelectedId,
    );
  }, [selectedFeatureId, data]);

  useEffect(() => {
    if (selectedRowIndex !== -1 && selectedFeatureId) {
      virtualizer.scrollToIndex(selectedRowIndex, {
        align: "start",
        behavior: "auto",
      });
    }
  }, [selectedRowIndex, selectedFeatureId, virtualizer]);

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
                <DropdownMenuItem onClick={handleSelectAllColumns}>
                  {t("Select All")}
                </DropdownMenuItem>
                <DropdownMenuItem onClick={handleDeselectAllColumns}>
                  {t("Deselect All")}
                </DropdownMenuItem>
                <DropdownMenuSeparator />
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
        className="h-full overflow-auto rounded-md bg-primary/40"
        style={{ contain: "paint", willChange: "transform" }}>
        <div
          className="w-full caption-bottom overflow-auto text-xs"
          style={{
            height: `${virtualizer.getTotalSize() + 32}px`,
          }}>
          <Table>
            <TableHeader>
              {table.getHeaderGroups().map((headerGroup) => (
                <TableRow key={headerGroup.id}>
                  {headerGroup.headers.map((header) => {
                    return (
                      <TableHead
                        key={header.id}
                        className={`${condensed ? "h-8" : "h-10"}`}
                        style={{
                          width: Math.min(
                            header.getSize(),
                            header.column.columnDef.maxSize || 400,
                          ),
                          maxWidth: header.column.columnDef.maxSize || 400,
                        }}>
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
                      data-state={isSelected ? "selected" : undefined}
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
                            className={`${condensed ? "px-2 py-[2px]" : "p-2"}`}
                            style={{
                              width: Math.min(
                                cell.column.getSize(),
                                cell.column.columnDef.maxSize || 400,
                              ),
                              maxWidth: cell.column.columnDef.maxSize || 400,
                            }}>
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
          </Table>
        </div>
      </div>
    </div>
  );
}

export { VirtualizedTable };
