import {
  Input,
  Pagination,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Asset } from "@flow/types";

import { AssetCard } from "./AssetCard";

type Props = {
  assets: Asset[];
  currentPage: number;
  totalPages: number;
  setAssetToBeDeleted: (asset: string | undefined) => void;
  setCurrentPage?: (page: number) => void;
  orderDirections: Record<string, string>;
  currentOrder?: string;
  handleOrderChange?: () => void;
};
const AssetsGridView: React.FC<Props> = ({
  assets,
  currentPage,
  totalPages,
  orderDirections,
  currentOrder,
  handleOrderChange,
  setAssetToBeDeleted,
  setCurrentPage,
}) => {
  const t = useT();
  return (
    <div className="overflow-y-auto">
      <div className="flex items-center gap-4 py-3">
        <Input
          placeholder={t("Search") + "..."}
          // value={globalFilter ?? ""}
          // onChange={(e) => setGlobalFilter(String(e.target.value))}
          className="max-w-sm"
        />

        {currentOrder && (
          <Select
            value={currentOrder || "DESC"}
            onValueChange={handleOrderChange}>
            <SelectTrigger className="h-[32px] w-[100px]">
              <SelectValue placeholder={orderDirections.ASC} />
            </SelectTrigger>
            <SelectContent>
              {Object.entries(orderDirections).map(([value, label]) => (
                <SelectItem key={value} value={value}>
                  {label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      </div>

      <div className="grid min-w-0 grid-cols-1 gap-2 pb-2 sm:grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
        {assets.map((a) => (
          <AssetCard
            key={a.id}
            asset={a}
            setAssetToBeDeleted={setAssetToBeDeleted}
          />
        ))}
      </div>
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
