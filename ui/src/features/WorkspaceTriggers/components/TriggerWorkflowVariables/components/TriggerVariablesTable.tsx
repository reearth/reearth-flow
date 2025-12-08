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

import { VariableMapping } from "..";

type Props = {
  className?: string;
  projectVariables: VariableMapping[];
  columns: ColumnDef<VariableMapping, unknown>[];
};

const TriggerVariablesTable: React.FC<Props> = ({
  className,
  projectVariables,
  columns,
}) => {
  const t = useT();
  const table = useReactTable({
    data: projectVariables,
    columns,
    getCoreRowModel: getCoreRowModel(),
    columnResizeMode: "onChange",
  });

  const { rows } = table.getRowModel();
  return (
    <div className="max-h-[50vh] overflow-auto">
      <Table className={`rounded-md bg-inherit ${className}`}>
        <TableHeader className="sticky top-0 z-10 bg-background/50 backdrop-blur-2xl">
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow
              key={headerGroup.id}
              className="bg-background/50 backdrop-blur-2xl">
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
          {rows.length > 0 ? (
            rows.map((row) => (
              <TableRow
                key={row.id}
                className="bg-secondary"
                data-state={row.getIsSelected() ? "selected" : undefined}>
                {row.getVisibleCells().map((cell: any) => (
                  <TableCell key={cell.id} className={`p-2`}>
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </TableCell>
                ))}
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell
                colSpan={columns.length + 1}
                className="h-24 text-center">
                {t("No Results")}
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
};

export { TriggerVariablesTable };
