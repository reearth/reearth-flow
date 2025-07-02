import {
  FlowLogo,
  Input,
  LoadingSkeleton,
  Pagination,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { useT } from "@flow/lib/i18n";
import type { Asset } from "@flow/types";

import { AssetCard } from "./AssetCard";

type Props = {
  assets?: Asset[];
  isFetching: boolean;
  currentPage: number;
  totalPages: number;
  setAssetToBeDeleted: (asset: string | undefined) => void;
  setCurrentPage?: (page: number) => void;
  sortOptions: { value: string; label: string }[];
  currentSortValue: string;
  handleOrderChange: (value: string) => void;
  searchTerm?: string;
  setSearchTerm: (term: string) => void;
};
const AssetsGridView: React.FC<Props> = ({
  assets,
  isFetching,
  currentPage,
  totalPages,
  sortOptions,
  currentSortValue,
  handleOrderChange,
  setAssetToBeDeleted,
  setCurrentPage,
  searchTerm,
  setSearchTerm,
}) => {
  const t = useT();

  return (
    <div className="overflow-y-auto">
      <div className="flex items-center gap-4 py-3">
        <Input
          placeholder={t("Search") + "..."}
          value={searchTerm ?? ""}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="max-w-sm"
        />

        <Select value={currentSortValue} onValueChange={handleOrderChange}>
          <SelectTrigger className="h-[32px] w-[150px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {sortOptions.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      {isFetching ? (
        <LoadingSkeleton />
      ) : assets && assets.length > 0 ? (
        <div className="grid min-w-0 grid-cols-1 gap-2 pb-2 sm:grid-cols-3 lg:grid-cols-3 xl:grid-cols-3 2xl:grid-cols-4">
          {assets?.map((a) => (
            <AssetCard
              key={a.id}
              asset={a}
              setAssetToBeDeleted={setAssetToBeDeleted}
            />
          ))}
        </div>
      ) : (
        <BasicBoiler
          text={t("No Assets")}
          icon={<FlowLogo className="mt-3 mb-3 size-16 text-accent" />}
        />
      )}
      <div className="mb-3">
        <Pagination
          currentPage={currentPage}
          setCurrentPage={setCurrentPage}
          totalPages={totalPages}
        />
      </div>
    </div>
  );
};

export { AssetsGridView };
