import { FileIcon } from "@phosphor-icons/react";

import { Card, CardContent, CardFooter, CardHeader } from "@flow/components";
import { Icon } from "@flow/components/Icon";
import { getIconFileType } from "@flow/features/AssetsDialog/components/utils";
import type { CmsAsset } from "@flow/types/cmsIntegration";
import { formatDate, formatFileSize } from "@flow/utils";

import { extensionOf } from "../../utils";

type Props = {
  asset: CmsAsset;
  projectName?: string;
  formatLabel: string;
  selected: boolean;
  onClick: (asset: CmsAsset) => void;
  onDoubleClick: (asset: CmsAsset) => void;
};

const CmsAssetCard: React.FC<Props> = ({
  asset,
  projectName,
  formatLabel,
  selected,
  onClick,
  onDoubleClick,
}) => {
  const fileIcon = getIconFileType(extensionOf(asset.filename));

  return (
    <Card
      className={`group relative cursor-pointer bg-card hover:border-border ${selected ? "border-primary ring-1 ring-primary" : "border-transparent"}`}
      title={asset.filename}
      role="button"
      tabIndex={0}
      aria-pressed={selected}
      onKeyDown={(e) => {
        if (e.key !== "Enter" && e.key !== " ") return;
        e.preventDefault();
        onClick(asset);
      }}
      onClick={() => onClick(asset)}
      onDoubleClick={() => onDoubleClick(asset)}>
      <CardContent className="relative flex items-start justify-center p-2">
        {fileIcon ? (
          <Icon
            icon={fileIcon}
            size={70}
            className="opacity-50 group-hover:opacity-90"
          />
        ) : (
          <FileIcon
            weight="thin"
            className="size-17.5 opacity-50 group-hover:opacity-90"
          />
        )}
        <span className="absolute top-1 right-1 rounded bg-secondary px-1.5 py-0.5 text-[10px] text-muted-foreground">
          {formatLabel}
        </span>
      </CardContent>
      <CardHeader className="px-1 py-0.5">
        {projectName && (
          <p className="truncate text-[10px] text-zinc-400 dark:font-thin">
            {projectName}
          </p>
        )}
        <p className="truncate text-xs dark:font-extralight">
          {asset.filename}
        </p>
      </CardHeader>
      <CardFooter className="flex flex-col items-start px-1 pb-0.5">
        <p className="text-xs text-zinc-400 dark:font-thin">
          {formatDate(asset.createdAt)}
        </p>
        <p className="text-xs text-zinc-400 dark:font-thin">
          {formatFileSize(asset.size)}
        </p>
      </CardFooter>
    </Card>
  );
};

export default CmsAssetCard;
