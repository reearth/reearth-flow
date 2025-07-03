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
  sortOptions: { value: string; label: string }[];
  currentSortValue: string;
  searchTerm?: string;
  setAssetToBeDeleted: (asset: string | undefined) => void;
  setCurrentPage?: (page: number) => void;
  setSearchTerm: (term: string) => void;
  onSortChange: (value: string) => void;
  onCopyUrlToClipBoard: (url: string) => void;
  onAssetDownload: (
    e: React.MouseEvent<HTMLAnchorElement>,
    asset: Asset,
  ) => void;
};
const AssetsGridView: React.FC<Props> = ({
  assets,
  isFetching,
  currentPage,
  totalPages,
  sortOptions,
  currentSortValue,
  searchTerm,
  setAssetToBeDeleted,
  setCurrentPage,
  setSearchTerm,
  onSortChange,
  onCopyUrlToClipBoard,
  onAssetDownload,
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
        <Select value={currentSortValue} onValueChange={onSortChange}>
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
        <LoadingSkeleton className="mt-40" />
      ) : assets && assets.length > 0 ? (
        <div className="grid min-w-0 grid-cols-1 gap-2 pb-2 sm:grid-cols-3 lg:grid-cols-3 xl:grid-cols-3 2xl:grid-cols-4">
          {assets?.map((a) => (
            <AssetCard
              key={a.id}
              asset={a}
              onCopyUrlToClipBoard={onCopyUrlToClipBoard}
              onAssetDownload={onAssetDownload}
              setAssetToBeDeleted={setAssetToBeDeleted}
            />
          ))}
        </div>
      ) : (
        <BasicBoiler
          text={t("No Assets")}
          icon={<FlowLogo className="mt-40 mb-3 size-16 text-accent" />}
        />
      )}
      {assets && assets.length > 0 && (
        <div className="mb-3">
          <Pagination
            currentPage={currentPage}
            setCurrentPage={setCurrentPage}
            totalPages={totalPages}
          />
        </div>
      )}
    </div>
  );
};

export { AssetsGridView };
