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
  surpressAutoScroll?: boolean;
  onRowClick?: (row: TData) => void;
  onRowDoubleClick?: (row: TData) => void;
  customGlobalFilter?: (
    row: any,
    _columnId: string,
    filterValue: string,
  ) => boolean;
  setSearchTerm?: (term: string) => void;
};

function VirtualizedTable<TData, TValue>({
  columns,
  data,
  selectColumns = false,
  showFiltering = false,
  condensed,
  selectedFeatureId,
  surpressAutoScroll,
  searchTerm,
  onRowClick,
  onRowDoubleClick,
  customGlobalFilter,
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
    globalFilterFn: customGlobalFilter ? customGlobalFilter : undefined,
    onGlobalFilterChange: setGlobalFilter,
    getFilteredRowModel: getFilteredRowModel(),
    state: {
      sorting,
      columnVisibility,
      rowSelection,
      globalFilter: searchTerm || globalFilter,
    },
    manualPagination: true,
  });

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
    estimateSize: () => (condensed ? 24 : 34),
  });

  const isItemRenderedAndInView = useCallback(
    (index: number) => {
      const scrollElement = parentRef.current;
      if (!scrollElement) return false;

      const viewportTop = scrollElement.scrollTop;
      const viewportBottom = viewportTop + scrollElement.clientHeight;

      const virtualItem = virtualizer
        .getVirtualItems()
        .find((item) => item.index === index);

      if (!virtualItem) return false;

      const itemTop = virtualItem.start;
      const itemBottom = virtualItem.start + virtualItem.size;

      return itemTop >= viewportTop && itemBottom <= viewportBottom;
    },
    [virtualizer],
  );

  const [parentHeight, setParentHeight] = useState<number>(0);

  useEffect(() => {
    if (!parentRef.current) return;
    const el = parentRef.current;

    const ro = new ResizeObserver(() => {
      setParentHeight(el.clientHeight);
    });

    ro.observe(el);
    setParentHeight(el.clientHeight);

    return () => ro.disconnect();
  }, []);

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
    if (selectedRowIndex === -1 || surpressAutoScroll) return;

    if (!isItemRenderedAndInView(selectedRowIndex)) {
      virtualizer.scrollToIndex(selectedRowIndex, {
        align: "start",
        behavior: "auto",
      });
    }
  }, [
    selectedRowIndex,
    surpressAutoScroll,
    isItemRenderedAndInView,
    virtualizer,
  ]);

  const totalSize = virtualizer.getTotalSize();
  const spacerHeight = Math.max(totalSize, parentHeight);

  return (
    <div className="flex h-full min-h-0 flex-col">
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
        className="min-h-0 flex-1 overflow-auto rounded-md bg-primary/40"
        style={{ contain: "paint", willChange: "transform" }}>
        <div
          className="relative w-full"
          style={{
            height: `${spacerHeight}px`,
          }}>
          <Table className="w-full text-xs">
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
                      className="after:border-line-200 relative cursor-pointer border-0 after:absolute after:top-0 after:left-0 after:z-10 after:w-full after:border-b"
                      style={{
                        height: `${virtualRow.size}px`,
                        transform: `translateY(${
                          virtualRow.start - idx * virtualRow.size
                        }px)`,
                      }}
                      data-state={isSelected ? "selected" : undefined}
                      onClick={() => {
                        row.toggleSelected();
                        onRowClick?.(row.original);
                      }}
                      onDoubleClick={() => onRowDoubleClick?.(row.original)}>
                      {row.getVisibleCells().map((cell: any) => (
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
  );
}

export { VirtualizedTable };
