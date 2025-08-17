import { Skeleton } from "../Skeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../Table";

import PaginationSkeleton from "./PaginationSkeleton";

const LoadingTableSkeleton: React.FC<{
  columns: number;
  rows: number;
  className?: string;
  condensed?: boolean;
  hasFilter?: boolean;
  hasQuickActions?: boolean;
  hasOrdering?: boolean;
  hasColumns?: boolean;
  hasPagination?: boolean;
}> = ({
  columns,
  rows,
  condensed,
  hasQuickActions,
  hasFilter,
  hasOrdering,
  hasColumns,
  hasPagination,
}) => {
  return (
    <div className="flex h-full flex-col justify-between">
      {(hasFilter || hasOrdering || hasColumns) && (
        <div
          className={`flex items-center justify-between gap-4 ${condensed ? "py-1" : "py-3"}`}>
          <div className="flex items-center gap-2">
            {hasFilter && <Skeleton className="h-8 w-[300px]" />}
            {hasOrdering && <Skeleton className="h-8 w-[100px]" />}
          </div>
          {hasColumns && <Skeleton className="h-8 w-[76px]" />}
        </div>
      )}

      <div className="flex-1 overflow-auto">
        <div className="overflow-auto rounded-md border">
          <Table className="mx-auto">
            <TableHeader>
              <TableRow>
                <TableHead className={`${condensed ? "h-8" : "h-10"}`}>
                  <Skeleton className="h-7 w-[30px]" />
                </TableHead>
                {[...Array(columns - 1)].map((_, i) => (
                  <TableHead
                    key={i}
                    className={`${condensed ? "h-8" : "h-10"}`}>
                    <Skeleton className="h-7 w-[100px]" />
                  </TableHead>
                ))}
              </TableRow>
            </TableHeader>
            <TableBody>
              {[...Array(rows)].map((_, i) => (
                <TableRow key={i}>
                  {[...Array(columns)].map((_, j) => (
                    <TableCell
                      key={`${i}-${j}`}
                      className={`${condensed ? "px-2 py-[2px]" : "p-2"} ${hasQuickActions ? "h-13" : ""}`}>
                      <Skeleton className="h-5 w-[175px]" />
                    </TableCell>
                  ))}
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </div>
      {hasPagination && <PaginationSkeleton />}
    </div>
  );
};

export default LoadingTableSkeleton;
