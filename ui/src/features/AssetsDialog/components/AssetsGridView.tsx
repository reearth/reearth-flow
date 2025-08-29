import { FlowLogo, LoadingSkeleton } from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { useT } from "@flow/lib/i18n";
import type { Asset } from "@flow/types";

import { AssetCard } from "./AssetCard";

type Props = {
  assets?: Asset[];
  isFetching: boolean;
  isDebouncingSearch?: boolean;
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
  isFetching,
  isDebouncingSearch,
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
        {isDebouncingSearch || isFetching ? (
          <LoadingSkeleton className="flex h-full justify-center" />
        ) : assets && assets.length > 0 ? (
          <div className="grid min-w-0 grid-cols-5 gap-2 p-2">
            {assets?.map((a) => (
              <AssetCard
                key={a.id}
                asset={a}
                isDeleting={isDeleting}
                onCopyUrlToClipBoard={onCopyUrlToClipBoard}
                onAssetDownload={onAssetDownload}
                setAssetToBeDeleted={setAssetToBeDeleted}
                setAssetToBeEdited={setAssetToBeEdited}
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
