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
import { useEffect, useState } from "react";

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
import { useT } from "@flow/providers";
import { Run } from "@flow/types";

type Props = {
  runs: Run[];
  rowSelection?: RowSelectionState;
  selectedRun?: Run;
  onRunSelect?: (run: Run) => void;
};

const RunsTable: React.FC<Props> = ({ runs, selectedRun, onRunSelect }) => {
  const t = useT();
  const [sorting, setSorting] = useState<SortingState>([
    { id: "completedAt", desc: true },
    { id: "startedAt", desc: true },
  ]);
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});
  const [rowSelection, setRowSelection] = useState({});
  const [globalFilter, setGlobalFilter] = useState("");

  const columns: ColumnDef<Run>[] = [
    {
      accessorKey: "id",
      header: t("ID"),
      size: 20,
      minSize: 10,
      maxSize: 30,
    },
    {
      accessorKey: "project.name",
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
    data: runs,
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

  useEffect(() => {
    if (rowSelection) {
      const selected = table
        ?.getRowModel()
        .rows.filter(r => r.getIsSelected().valueOf())[0]?.original;
      if (selected !== selectedRun) {
        onRunSelect?.(selected);
      }
    }
  }, [rowSelection, selectedRun, table, onRunSelect]);

  return (
    <div className="flex flex-col gap-4 py-4">
      <div className="flex items-center gap-4">
        <Input
          placeholder={t("Search") + "..."}
          value={globalFilter ?? ""}
          onChange={e => setGlobalFilter(String(e.target.value))}
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
              .filter(column => column.getCanHide())
              .map(column => {
                return (
                  <DropdownMenuCheckboxItem
                    key={column.id}
                    className="capitalize"
                    checked={column.getIsVisible()}
                    onCheckedChange={value => column.toggleVisibility(!!value)}>
                    {column.id}
                  </DropdownMenuCheckboxItem>
                );
              })}
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
      <div className="rounded-md border border-zinc-700">
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map(headerGroup => (
              <TableRow key={headerGroup.id} className="hover:bg-transparent">
                {headerGroup.headers.map(header => {
                  return (
                    <TableHead key={header.id}>
                      {header.isPlaceholder
                        ? null
                        : flexRender(header.column.columnDef.header, header.getContext())}
                    </TableHead>
                  );
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map(row => (
                <TableRow
                  key={row.id}
                  className="cursor-pointer"
                  data-state={row.getIsSelected() && "selected"}
                  onClick={() => row.toggleSelected()}
                  onSelect={s => console.log("S", s)}>
                  {row.getVisibleCells().map(cell => (
                    <TableCell key={cell.id}>
                      {flexRender(cell.column.columnDef.cell, cell.getContext())}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={columns.length} className="h-24 text-center">
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

export { RunsTable };
