import {
  ClipboardTextIcon,
  DotsThreeVerticalIcon,
  DownloadIcon,
  FileIcon,
  PencilIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { useState } from "react";

import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Asset } from "@flow/types";

type Props = {
  asset: Asset;
  setAssetToBeDeleted: (asset: string | undefined) => void;
  setAssetToBeEdited: (asset: Asset | undefined) => void;
  onDoubleClick?: (asset: Asset) => void;
  onCopyUrlToClipBoard: (url: string) => void;
  onAssetDownload: (
    e: React.MouseEvent<HTMLAnchorElement>,
    asset: Asset,
  ) => void;
};

const AssetCard: React.FC<Props> = ({
  asset,
  onCopyUrlToClipBoard,
  onAssetDownload,
  setAssetToBeDeleted,
  setAssetToBeEdited,
  onDoubleClick,
}) => {
  const t = useT();
  const [persistOverlay, setPersistOverlay] = useState(false);

  const { id, name, createdAt, size, url } = asset;

  const handleDoubleClick = () => {
    if (onDoubleClick) {
      onDoubleClick(asset);
    }
  };

  return (
    <Card
      className="group relative cursor-pointer border-transparent bg-secondary hover:border-border"
      key={id}
      onDoubleClick={handleDoubleClick}>
      <CardContent className="flex items-start justify-center p-2">
        <FileIcon
          weight="thin"
          size={70}
          className="group:hover:opacity-90 opacity-50"
        />
      </CardContent>
      <CardHeader className="px-1 py-0.5">
        <CardTitle className="truncate text-xs dark:font-extralight">
          {name}
        </CardTitle>
      </CardHeader>
      <CardFooter className="flex flex-col items-start px-1 pb-0.5">
        <p className="text-xs text-zinc-400 dark:font-thin">{createdAt}</p>
        <p className="text-xs text-zinc-400 dark:font-thin">{size}</p>
      </CardFooter>
      <div
        className={`absolute inset-0 ${persistOverlay ? "flex flex-col" : "hidden"} rounded-lg group-hover:flex group-hover:flex-col`}>
        <div className="flex h-[75px] items-center justify-center rounded-t-lg bg-black/30 p-4" />
        <div className="flex flex-1 justify-end rounded-b-lg">
          <DropdownMenu
            modal={false}
            onOpenChange={(o) => setPersistOverlay(o)}>
            <DropdownMenuTrigger
              className="flex h-full w-[30px] items-center justify-center rounded-br-lg hover:bg-primary"
              onClick={(e) => e.stopPropagation()}>
              <DotsThreeVerticalIcon className="size-[24px]" />
            </DropdownMenuTrigger>
            <DropdownMenuContent
              align="center"
              onClick={(e) => e.stopPropagation()}>
              <DropdownMenuItem
                className="justify-between gap-2 text-warning"
                disabled={!url}
                onClick={() => setAssetToBeEdited(asset)}>
                {t("Edit Asset")}
                <PencilIcon weight="light" />
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className="justify-between gap-2"
                disabled={!url}
                onClick={() => onCopyUrlToClipBoard(url)}>
                {t("Copy Asset URL")}
                <ClipboardTextIcon weight="light" />
              </DropdownMenuItem>
              <a href={url} onClick={(e) => onAssetDownload(e, asset)}>
                <DropdownMenuItem
                  className="justify-between gap-2"
                  disabled={!url}>
                  {t("Download Asset")}
                  <DownloadIcon weight="light" />
                </DropdownMenuItem>
              </a>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className="justify-between gap-4 text-destructive"
                onClick={(e) => {
                  e.stopPropagation();
                  setAssetToBeDeleted(id);
                }}>
                {t("Delete Asset")}
                <TrashIcon weight="light" />
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </Card>
  );
};

export { AssetCard };
