import { CaretLeft, CaretRight } from "@phosphor-icons/react";
import * as React from "react";

import { IconButton } from "../buttons";

type PaginationProps = {
  hasNextPage?: boolean;
  onNextPage?: () => void;
  onPrevPage?: () => void;
  currentPage?: number;
  totalPages?: number;
  isFetchingNextPage?: boolean;
};
const Pagination: React.FC<PaginationProps> = ({
  hasNextPage,
  onNextPage,
  onPrevPage,
  currentPage = 0,
  totalPages = 0,
  isFetchingNextPage,
}) => {
  return (
    <div className="flex justify-center gap-4 pt-4">
      <div className="flex gap-1">
        <IconButton
          className="rounded border p-1"
          variant="outline"
          icon={<CaretLeft />}
          onClick={onPrevPage}
          disabled={currentPage === 0}
        />
        <div className="flex min-w-10 items-center justify-center gap-1">
          <p className="text-sm font-light">{currentPage + 1}</p>
          <p className="text-xs font-light">/</p>
          <p className="text-sm font-light">{totalPages}</p>
        </div>
        <IconButton
          className="rounded border p-1"
          variant="outline"
          icon={<CaretRight />}
          onClick={onNextPage}
          disabled={!hasNextPage || isFetchingNextPage}
        />
      </div>
    </div>
  );
};

export { Pagination };
