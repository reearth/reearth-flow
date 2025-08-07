import {
  CaretDoubleLeftIcon,
  CaretDoubleRightIcon,
  CaretLeftIcon,
  CaretRightIcon,
} from "@phosphor-icons/react";

import { IconButton } from "../buttons";
import { Skeleton } from "../Skeleton";

const PaginationSkeleton: React.FC = () => {
  return (
    <div className="flex justify-center gap-4 pt-2">
      <div className="flex gap-1">
        <IconButton variant="outline" icon={<CaretDoubleLeftIcon />} disabled />
        <IconButton variant="outline" icon={<CaretLeftIcon />} disabled />
        <Skeleton className=" h-[36px] w-[40px] rounded" />

        <IconButton
          className="rounded border p-1"
          icon={<CaretRightIcon />}
          disabled
        />

        <IconButton
          className="rounded border p-1"
          icon={<CaretDoubleRightIcon />}
          disabled
        />
      </div>
    </div>
  );
};

export default PaginationSkeleton;
