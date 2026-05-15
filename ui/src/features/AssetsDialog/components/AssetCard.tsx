import {
  ClipboardTextIcon,
  DotsThreeVerticalIcon,
  DownloadIcon,
  FileIcon,
  PencilLineIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { useMemo, useState } from "react";

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
import { Icon, IconName } from "@flow/components/Icon";
import { useT } from "@flow/lib/i18n";
import { Asset } from "@flow/types";

type Props = {
  asset: Asset;
  readonly?: boolean;
  isDeleting?: boolean;
  onCopyUrlToClipBoard: (url: string) => void;
  onAssetDownload: (
    e: React.MouseEvent<HTMLAnchorElement>,
    asset: Asset,
  ) => void;
  setAssetToBeDeleted: (asset: string | undefined) => void;
  setAssetToBeEdited: (asset: Asset | undefined) => void;
  onDoubleClick?: (asset: Asset) => void;
};

const AssetCard: React.FC<Props> = ({
  asset,
  readonly,
  isDeleting,
  onCopyUrlToClipBoard,
  onAssetDownload,
  setAssetToBeDeleted,
  setAssetToBeEdited,
  onDoubleClick,
}) => {
  const t = useT();
  const [persistOverlay, setPersistOverlay] = useState(false);

  const { id, name, fileName, createdAt, size, url } = asset;

  const ext = fileName.split(".").pop()?.toLowerCase();

  const handleDoubleClick = () => {
    if (onDoubleClick) {
      onDoubleClick(asset);
    }
  };
  const iconType = useMemo((): IconName | undefined => {
    switch (ext) {
      case "csv":
        return "fileCSV";
      case "czml":
        return "fileCzml";
      case "geojson":
        return "fileGeoJSON";
      case "glb":
        return "fileGlb";
      case "gltf":
        return "fileGltf";
      case "gml":
        return "fileGml";
      case "gpkg":
        return "fileGpkg";
      case "jpg":
        return "fileJpg";
      case "jpeg":
        return "fileJpeg";
      case "json":
        return "fileJson";
      case "mtl":
        return "fileMtl";
      case "obj":
        return "fileObj";
      case "png":
        return "filePng";
      case "py":
        return "filePy";
      case "tif":
        return "fileTif";
      case "tiff":
        return "fileTiff";
      case "tsv":
        return "fileTsv";
      case "zip":
        return "fileZip";
      default:
        return undefined;
    }
  }, [ext]);

  return (
    <Card
      className="group relative cursor-pointer border-transparent bg-card hover:border-border"
      key={id}
      onDoubleClick={readonly ? undefined : handleDoubleClick}>
      <CardContent className="flex items-start justify-center p-2">
        {iconType ? (
          <Icon
            icon={iconType}
            size={70}
            className="opacity-50 group-hover:opacity-90"
          />
        ) : (
          <FileIcon
            weight="thin"
            className="size-17.5 opacity-50 group-hover:opacity-90"
          />
        )}
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
                disabled={isDeleting || !url || readonly}
                onClick={() => setAssetToBeEdited(asset)}>
                {t("Edit Asset")}
                <PencilLineIcon weight="light" />
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className="justify-between gap-2"
                disabled={isDeleting || !url}
                onClick={() => onCopyUrlToClipBoard(url)}>
                {t("Copy Asset URL")}
                <ClipboardTextIcon weight="light" />
              </DropdownMenuItem>
              <a href={url} onClick={(e) => onAssetDownload(e, asset)}>
                <DropdownMenuItem
                  className="justify-between gap-2"
                  disabled={isDeleting || !url}>
                  {t("Download Asset")}
                  <DownloadIcon weight="light" />
                </DropdownMenuItem>
              </a>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className="justify-between gap-4 text-destructive"
                disabled={isDeleting || readonly}
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
