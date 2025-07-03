import {
  ClipboardTextIcon,
  DotsThreeVerticalIcon,
  DownloadIcon,
  FileIcon,
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
  onCopyUrlToClipBoard: (url: string) => void;
  onAssetDownload: (
    e: React.MouseEvent<HTMLAnchorElement>,
    asset: Asset,
  ) => void;
  setAssetToBeDeleted: (asset: string | undefined) => void;
};

const AssetCard: React.FC<Props> = ({
  asset,
  onCopyUrlToClipBoard,
  onAssetDownload,
  setAssetToBeDeleted,
}) => {
  const t = useT();
  const [persistOverlay, setPersistOverlay] = useState(false);

  const { id, name, createdAt, url } = asset;

  return (
    <Card
      className="group relative cursor-pointer border-transparent bg-secondary hover:border-border"
      key={id}>
      <CardContent className="relative flex h-[80px] items-center justify-center p-0">
        <FileIcon className="group:hover:opacity-90 size-[80px]  translate-x-20 opacity-50" />
      </CardContent>
      <CardHeader className="px-2 py-1">
        <CardTitle className="truncate dark:font-extralight">{name}</CardTitle>
      </CardHeader>
      <CardFooter className="flex px-2 pb-1">
        <p className="text-xs text-zinc-400 dark:font-thin">
          {t("Uploaded At:")} {createdAt}
        </p>
      </CardFooter>
      <div
        className={`absolute inset-0 ${persistOverlay ? "flex flex-col" : "hidden"} rounded-lg group-hover:flex group-hover:flex-col`}>
        <div className="flex h-[80px] items-center justify-center rounded-t-lg bg-black/30 p-4 backdrop-blur-xs" />
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
              align="end"
              onClick={(e) => e.stopPropagation()}>
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
