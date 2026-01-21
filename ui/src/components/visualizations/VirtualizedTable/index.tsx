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
import { Virtualizer } from "@tanstack/react-virtual";
import { RefObject, useCallback, useEffect, useState } from "react";

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
  parentRef: RefObject<HTMLDivElement | null>;
  virtualizer: Virtualizer<HTMLDivElement, Element>;
  selectColumns?: boolean;
  showFiltering?: boolean;
  condensed?: boolean;
  searchTerm?: string;
  selectedRowIndex: number;
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
  parentRef,
  virtualizer,
  selectColumns = false,
  showFiltering = false,
  condensed,
  selectedRowIndex,
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

  const { rows } = table.getRowModel();

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
  }, [parentRef]);

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
                      className="after:border-line-200 after:absolute after:top-0 after:left-0 after:z-10 after:w-full after:border-b relative cursor-pointer border-0"
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
