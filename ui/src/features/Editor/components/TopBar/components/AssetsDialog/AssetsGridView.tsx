import { FlowLogo, LoadingSkeleton } from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { useT } from "@flow/lib/i18n";
import type { Asset } from "@flow/types";

import { AssetCard } from "./AssetCard";

type Props = {
  assets?: Asset[];
  isFetching: boolean;
  setAssetToBeDeleted: (asset: string | undefined) => void;
  setAssetToBeEdited: (asset: Asset | undefined) => void;
  onCopyUrlToClipBoard: (url: string) => void;
  onAssetDownload: (
    e: React.MouseEvent<HTMLAnchorElement>,
    asset: Asset,
  ) => void;
};
const AssetsGridView: React.FC<Props> = ({
  assets,
  isFetching,
  setAssetToBeDeleted,
  setAssetToBeEdited,
  onCopyUrlToClipBoard,
  onAssetDownload,
}) => {
  const t = useT();

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="overflow-y-auto">
        {isFetching ? (
          <LoadingSkeleton className="flex h-[500px] justify-center" />
        ) : assets && assets.length > 0 ? (
          <div className="grid min-w-0 grid-cols-5 gap-2 pb-2">
            {assets?.map((a) => (
              <AssetCard
                key={a.id}
                asset={a}
                onCopyUrlToClipBoard={onCopyUrlToClipBoard}
                onAssetDownload={onAssetDownload}
                setAssetToBeDeleted={setAssetToBeDeleted}
                setAssetToBeEdited={setAssetToBeEdited}
              />
            ))}
          </div>
        ) : (
          <div className="flex h-[500px] justify-center">
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
