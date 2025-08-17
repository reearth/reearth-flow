import { FlowLogo, Skeleton } from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { ASSET_FETCH_RATE } from "@flow/lib/gql/assets/useQueries";
import { useT } from "@flow/lib/i18n";
import type { Asset } from "@flow/types";

import { AssetCard } from "./AssetCard";

type Props = {
  assets?: Asset[];
  isDebouncing?: boolean;
  isFetching: boolean;
  isDeleting: boolean;
  setAssetToBeDeleted: (asset: string | undefined) => void;
  setAssetToBeEdited: (asset: Asset | undefined) => void;
  onCopyUrlToClipBoard: (url: string) => void;
  onAssetDownload: (
    e: React.MouseEvent<HTMLAnchorElement>,
    asset: Asset,
  ) => void;
  onAssetDoubleClick?: (asset: Asset) => void;
};
const AssetsGridView: React.FC<Props> = ({
  assets,
  isDebouncing,
  isFetching,
  isDeleting,
  setAssetToBeDeleted,
  setAssetToBeEdited,
  onCopyUrlToClipBoard,
  onAssetDownload,
  onAssetDoubleClick,
}) => {
  const t = useT();

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto">
        {isDebouncing || isFetching ? (
          <div className="grid min-w-0 grid-cols-5 gap-2 p-2">
            {Array.from({ length: ASSET_FETCH_RATE }).map((_, index) => (
              <div
                key={index}
                className="flex h-[142px] items-end rounded-lg bg-secondary">
                <div className="mb-1 flex h-[50px] w-[200px] flex-col justify-center gap-1 px-2">
                  <Skeleton className=" h-[20px] w-[120px] " />
                  <Skeleton className="h-[16px] w-[85px]" />
                  <Skeleton className="h-[16px] w-[48px]" />
                </div>
              </div>
            ))}
          </div>
        ) : assets && assets.length > 0 ? (
          <div className="grid min-w-0 grid-cols-5 gap-2 p-2">
            {assets?.map((a) => (
              <AssetCard
                key={a.id}
                asset={a}
                isDeleting={isDeleting}
                setAssetToBeDeleted={setAssetToBeDeleted}
                setAssetToBeEdited={setAssetToBeEdited}
                onCopyUrlToClipBoard={onCopyUrlToClipBoard}
                onAssetDownload={onAssetDownload}
                onDoubleClick={onAssetDoubleClick}
              />
            ))}
          </div>
        ) : (
          <div className="flex h-full items-center justify-center">
            <BasicBoiler
              text={t("No Assets")}
              icon={<FlowLogo className=" mb-3 size-16 text-accent" />}
            />
          </div>
        )}
      </div>
    </div>
  );
};

export { AssetsGridView };
