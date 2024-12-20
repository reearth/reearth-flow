import {
  ColumnDef,
  RowSelectionState,
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
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@flow/components/Table";
import { useT } from "@flow/lib/i18n";
import { Job } from "@flow/types";

type Props = {
  jobs: Job[];
  rowSelection?: RowSelectionState;
  onJobSelect?: (jobId: string) => void;
};

const JobsTable: React.FC<Props> = ({ jobs, onJobSelect }) => {
  const t = useT();
  const [sorting, setSorting] = useState<SortingState>([
    { id: "completedAt", desc: true },
    { id: "startedAt", desc: true },
  ]);

  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});
  const [rowSelection, setRowSelection] = useState({});
  const [globalFilter, setGlobalFilter] = useState("");

  const columns: ColumnDef<Job>[] = [
    {
      accessorKey: "id",
      header: t("ID"),
      size: 20,
    },
    {
      accessorKey: "deployment.projectName",
      header: t("Project Name"),
      size: 10,
    },
    {
      accessorKey: "status",
      header: t("Status"),
      size: 10,
    },
    {
      accessorKey: "logs",
      header: t("Logs"),
      size: 10,
    },
    {
      accessorKey: "startedAt",
      header: t("Started At"),
    },
    {
      accessorKey: "completedAt",
      header: t("Completed At"),
    },
    {
      accessorKey: "trigger",
      header: t("Trigger"),
    },
  ];

  const table = useReactTable({
    data: jobs,
    columns,
    enableMultiRowSelection: false,
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
    <div className="flex flex-col gap-4 py-4">
      <div className="flex items-center gap-4">
        <Input
          placeholder={t("Search") + "..."}
          value={globalFilter ?? ""}
          onChange={(e) => setGlobalFilter(String(e.target.value))}
          className="max-w-sm"
        />
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
      </div>
      <div>
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id} className="border-none">
                {headerGroup.headers.map((header) => {
                  return (
                    <TableHead key={header.id} className="h-8 dark:font-thin">
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
                  className="h-10 cursor-pointer transition-all hover:bg-primary"
                  data-state={row.getIsSelected() && "selected"}
                  onClick={() => {
                    row.toggleSelected();
                    onJobSelect?.(row.original.id);
                  }}
                  onSelect={(s) => console.log("S", s)}>
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
  );
};

export { JobsTable };
