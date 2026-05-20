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
import { AwarenessUser, WorkflowVariable } from "@flow/types";

type Props = {
  className?: string;
  workflowVariables: WorkflowVariable[];
  columns: ColumnDef<WorkflowVariable, unknown>[];
  onReorder?: (oldIndex: number, newIndex: number) => void;
  variableFocusMap?: Record<string, AwarenessUser[]>;
  variableEditMap?: Record<string, AwarenessUser[]>;
};

const SortableRow: React.FC<{
  row: any;
  variable: WorkflowVariable;
  focusedUsers: AwarenessUser[];
  editingUsers: AwarenessUser[];
}> = ({ row, variable, focusedUsers, editingUsers }) => {
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

  const firstFocusedUser = focusedUsers[0];
  const firstEditingUser = editingUsers[0];
  const indicatorUser = firstFocusedUser ?? firstEditingUser;

  return (
    <TableRow
      key={row.id}
      ref={setNodeRef}
      style={style}
      className="hover:bg-primary/50"
      {...attributes}>
      <TableCell className="w-10 p-0">
        <div className="flex items-center">
          {indicatorUser && (
            <div
              className="w-1 self-stretch rounded-l"
              style={{ backgroundColor: indicatorUser.color }}
            />
          )}
          <div
            className="flex cursor-grab touch-none items-center justify-center p-1 active:cursor-grabbing"
            {...listeners}>
            <DotsSixIcon size={16} className="text-muted-foreground" />
          </div>
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

const WorkflowVariablesTable: React.FC<Props> = ({
  className,
  workflowVariables,
  columns,
  onReorder,
  variableFocusMap = {},
  variableEditMap = {},
}) => {
  const t = useT();

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (active.id !== over?.id) {
      const oldIndex = workflowVariables.findIndex(
        (item) => item.id === active.id,
      );
      const newIndex = workflowVariables.findIndex(
        (item) => item.id === over?.id,
      );

      if (oldIndex !== -1 && newIndex !== -1) {
        onReorder?.(oldIndex, newIndex);
      }
    }
  };

  const table = useReactTable({
    data: workflowVariables,
    columns,
    getCoreRowModel: getCoreRowModel(),
    columnResizeMode: "onChange",
    state: {},
  });

  const items = workflowVariables.map((item) => item.id);

  if (!onReorder) {
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
                  colSpan={columns.length}
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

  return (
    <div className="max-h-[50vh] overflow-auto">
      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={handleDragEnd}>
        <Table className={`rounded-md bg-inherit ${className}`}>
          <TableHeader className="sticky top-0 z-10 bg-background/50 backdrop-blur-2xl">
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow
                key={headerGroup.id}
                className="bg-background/50 backdrop-blur-2xl">
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
                  const variable = workflowVariables[row.index];
                  return (
                    <SortableRow
                      key={row.id}
                      row={row}
                      variable={variable}
                      focusedUsers={variableFocusMap[variable.id] ?? []}
                      editingUsers={variableEditMap[variable.id] ?? []}
                    />
                  );
                })
              ) : (
                <TableRow>
                  <TableCell
                    colSpan={columns.length + 1}
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

export { WorkflowVariablesTable };
