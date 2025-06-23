import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { useState } from "react";

import {
  Checkbox,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { ProjectVariable } from "@flow/types";

type Props = {
  className?: string;
  projectVariables: ProjectVariable[];
  columns: ColumnDef<ProjectVariable, unknown>[];
  selectedRow?: number;
  onRowClick: (projectVariable: ProjectVariable) => void;
};

const ProjectVariablesTable: React.FC<Props> = ({
  className,
  projectVariables,
  columns,
  onRowClick,
}) => {
  const t = useT();

  const [rowSelection, setRowSelection] = useState({});

  // Create columns with selection column prepended
  const columnsWithSelection: ColumnDef<ProjectVariable>[] = [
    {
      id: "select",
      header: ({ table }) => (
        <Checkbox
          checked={table.getIsAllPageRowsSelected()}
          onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
          aria-label="Select all"
        />
      ),
      cell: ({ row }) => (
        <Checkbox
          checked={row.getIsSelected()}
          onCheckedChange={(value) => {
            row.toggleSelected(!!value);
            if (value) {
              onRowClick?.(row.original);
            }
          }}
          aria-label="Select row"
          onClick={(e) => e.stopPropagation()}
        />
      ),
      enableSorting: false,
      enableHiding: false,
    },
    ...columns,
  ];

  const table = useReactTable({
    data: projectVariables,
    columns: columnsWithSelection,
    enableMultiRowSelection: true,
    getCoreRowModel: getCoreRowModel(),
    // Sorting
    // onSortingChange: setSorting,
    // getSortedRowModel: getSortedRowModel(),
    // Visibility
    // onColumnVisibilityChange: setColumnVisibility,
    columnResizeMode: "onChange",
    // Row selection
    onRowSelectionChange: setRowSelection,
    // Filtering
    // onGlobalFilterChange: setGlobalFilter,
    // getFilteredRowModel: getFilteredRowModel(),
    // getPaginationRowModel: enablePagination
    //   ? getPaginationRowModel()
    //   : undefined,
    // onPaginationChange: setPagination,
    state: {
      //   sorting,
      //   columnVisibility,
      rowSelection,
      //   globalFilter,
      //   pagination,
    },
    // manualPagination: true,
  });

  return (
    <Table className={`rounded-md bg-inherit ${className}`}>
      <TableHeader>
        {table.getHeaderGroups().map((headerGroup) => (
          <TableRow key={headerGroup.id}>
            {headerGroup.headers.map((header) => {
              return (
                <TableHead key={header.id} className="h-8 whitespace-nowrap">
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
              className="hover:bg-primary/50 data-[state=selected]:bg-primary/50"
              data-state={row.getIsSelected() && "selected"}
              onClick={(e) => {
                // Only trigger onRowClick for visual feedback, don't toggle selection
                // Selection is now handled only by the checkbox
                const target = e.target as HTMLElement;
                // Don't trigger if clicking on interactive elements
                if (!target.closest('input, button, [role="button"]')) {
                  onRowClick?.(row.original);
                }
              }}>
              {row.getVisibleCells().map((cell) => (
                <TableCell key={cell.id} className="px-2 py-[2px]">
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </TableCell>
              ))}
            </TableRow>
          ))
        ) : (
          <TableRow>
            <TableCell
              colSpan={columnsWithSelection.length}
              className="h-24 text-center">
              {t("No Results")}
            </TableCell>
          </TableRow>
        )}
      </TableBody>
    </Table>
  );
};

export { ProjectVariablesTable };
