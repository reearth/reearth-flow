import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";

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
  selectedIndices: number[];
  onSelectionChange: (selectedIndices: number[]) => void;
};

const ProjectVariablesTable: React.FC<Props> = ({
  className,
  projectVariables,
  columns,
  selectedIndices,
  onSelectionChange,
}) => {
  const t = useT();

  // Convert selectedIndices array to rowSelection object format expected by React Table
  const rowSelection = selectedIndices.reduce(
    (acc, index) => {
      acc[index.toString()] = true;
      return acc;
    },
    {} as Record<string, boolean>,
  );

  const handleRowSelectionChange = (updater: any) => {
    const newRowSelection =
      typeof updater === "function" ? updater(rowSelection) : updater;
    const newSelectedIndices = Object.keys(newRowSelection)
      .filter((key) => newRowSelection[key])
      .map((key) => parseInt(key, 10));
    onSelectionChange(newSelectedIndices);
  };

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
    onRowSelectionChange: handleRowSelectionChange,
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

  console.log("asdfs", table.getRowModel().rows);

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
                // Selection is now handled only by the checkbox
                // Row clicks don't trigger selection anymore
                e.preventDefault();
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
