import {
  CaretDoubleLeftIcon,
  CaretDoubleRightIcon,
  CaretLeftIcon,
  CaretRightIcon,
} from "@phosphor-icons/react";
import * as React from "react";

import { IconButton } from "../buttons";

type PaginationProps = {
  currentPage: number;
  totalPages: number;
  setCurrentPage?: (page: number) => void;
};
const Pagination: React.FC<PaginationProps> = ({
  currentPage,
  totalPages,
  setCurrentPage,
}) => {
  return (
    <div className="flex justify-center gap-4 pt-2">
      <div className="flex gap-1">
        <IconButton
          variant="outline"
          icon={<CaretDoubleLeftIcon />}
          onClick={() => {
            if (currentPage > 1) {
              setCurrentPage?.(1);
              // table.setPageIndex(0);
            }
          }}
          disabled={currentPage <= 1}
        />
        <IconButton
          variant="outline"
          icon={<CaretLeftIcon />}
          onClick={() => {
            if (currentPage > 1) {
              setCurrentPage?.(currentPage - 1);
              // table.previousPage();
            }
          }}
          disabled={currentPage <= 1}
        />
        <div className="flex min-w-10 items-center justify-center gap-1">
          <p className="text-sm font-light">{currentPage}</p>
          <p className="text-xs font-light">/</p>
          <p className="text-sm font-light">{totalPages}</p>
        </div>
        <IconButton
          className="rounded border p-1"
          icon={<CaretRightIcon />}
          onClick={() => {
            if (currentPage < totalPages) {
              setCurrentPage?.(currentPage + 1);
              // table.nextPage();
            }
          }}
          disabled={currentPage >= totalPages}
        />

        <IconButton
          className="rounded border p-1"
          icon={<CaretDoubleRightIcon />}
          onClick={() => {
            if (currentPage < totalPages) {
              setCurrentPage?.(totalPages);
              // table.setPageIndex(totalPages - 1);
            }
          }}
          disabled={currentPage >= totalPages}
        />
      </div>
    </div>
  );
};

export { Pagination };
