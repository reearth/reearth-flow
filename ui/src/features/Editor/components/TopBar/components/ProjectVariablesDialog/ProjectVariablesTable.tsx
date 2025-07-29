import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
  useSortable,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { DotsSixIcon } from "@phosphor-icons/react";
import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";

import {
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
  onReorder?: (oldIndex: number, newIndex: number) => void;
};

// Sortable Row Component
const SortableRow: React.FC<{
  row: any;
  variable: ProjectVariable;
}> = ({ row, variable }) => {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: variable.id,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <TableRow
      key={row.id}
      ref={setNodeRef}
      style={style}
      className="hover:bg-primary/50"
      {...attributes}>
      <TableCell className="w-10">
        <div
          className="flex cursor-grab touch-none items-center justify-center p-1 active:cursor-grabbing"
          {...listeners}>
          <DotsSixIcon size={16} className="text-muted-foreground" />
        </div>
      </TableCell>
      {row.getVisibleCells().map((cell: any) => (
        <TableCell key={cell.id} className="px-2 py-[2px]">
          {flexRender(cell.column.columnDef.cell, cell.getContext())}
        </TableCell>
      ))}
    </TableRow>
  );
};

const ProjectVariablesTable: React.FC<Props> = ({
  className,
  projectVariables,
  columns,
  onReorder,
}) => {
  const t = useT();

  // Set up drag and drop sensors
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  );

  // Handle drag end
  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (active.id !== over?.id) {
      const oldIndex = projectVariables.findIndex(
        (item) => item.id === active.id,
      );
      const newIndex = projectVariables.findIndex(
        (item) => item.id === over?.id,
      );

      if (oldIndex !== -1 && newIndex !== -1) {
        onReorder?.(oldIndex, newIndex);
      }
    }
  };

  // We'll handle the drag handle separately in the row rendering
  const tableColumns = columns;

  const table = useReactTable({
    data: projectVariables,
    columns: tableColumns,
    getCoreRowModel: getCoreRowModel(),
    // Sorting
    // onSortingChange: setSorting,
    // getSortedRowModel: getSortedRowModel(),
    // Visibility
    // onColumnVisibilityChange: setColumnVisibility,
    columnResizeMode: "onChange",
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
      //   globalFilter,
      //   pagination,
    },
    // manualPagination: true,
  });

  const items = projectVariables.map((item) => item.id);

  if (!onReorder) {
    // Render table without drag and drop if onReorder is not provided
    return (
      <div className="max-h-[50vh] overflow-auto">
        <Table className={`rounded-md bg-inherit ${className}`}>
          <TableHeader className="sticky top-0 z-10 bg-background">
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <TableHead
                    key={header.id}
                    className="h-8 bg-background whitespace-nowrap">
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                          header.column.columnDef.header,
                          header.getContext(),
                        )}
                  </TableHead>
                ))}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map((row) => (
                <TableRow key={row.id} className="hover:bg-primary/50">
                  {row.getVisibleCells().map((cell) => (
                    <TableCell key={cell.id} className="px-2 py-[2px]">
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
                  colSpan={tableColumns.length}
                  className="h-24 text-center">
                  {t("No Results")}
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    );
  }

  // Render table with drag and drop
  return (
    <div className="max-h-[50vh] overflow-auto">
      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={handleDragEnd}>
        <Table className={`rounded-md bg-inherit ${className}`}>
          <TableHeader className="sticky top-0 z-10 bg-background/50">
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                <TableHead className="w-10" />
                {headerGroup.headers.map((header) => (
                  <TableHead key={header.id} className="h-8 whitespace-nowrap">
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                          header.column.columnDef.header,
                          header.getContext(),
                        )}
                  </TableHead>
                ))}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            <SortableContext
              items={items}
              strategy={verticalListSortingStrategy}>
              {table.getRowModel().rows?.length ? (
                table.getRowModel().rows.map((row) => {
                  const variable = projectVariables[row.index];
                  return (
                    <SortableRow key={row.id} row={row} variable={variable} />
                  );
                })
              ) : (
                <TableRow>
                  <TableCell
                    colSpan={tableColumns.length + 1}
                    className="h-24 text-center">
                    {t("No Results")}
                  </TableCell>
                </TableRow>
              )}
            </SortableContext>
          </TableBody>
        </Table>
      </DndContext>
    </div>
  );
};

export { ProjectVariablesTable };
