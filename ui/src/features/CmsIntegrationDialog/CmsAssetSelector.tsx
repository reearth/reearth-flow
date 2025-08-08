import {
  FileIcon,
  DotsThreeVerticalIcon,
  ClipboardTextIcon,
} from "@phosphor-icons/react";
import { useCallback, useState } from "react";

import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  ScrollArea,
} from "@flow/components";
import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import type { CmsItem, CmsModel } from "@flow/types/cmsIntegration";
import { copyToClipboard } from "@flow/utils/copyToClipboard";

type Props = {
  cmsItem: CmsItem;
  cmsModel: CmsModel;
  onAssetSelect: (assetUrl: string) => void;
};

const CmsAssetSelector: React.FC<Props> = ({
  cmsItem,
  cmsModel,
  onAssetSelect,
}) => {
  const t = useT();
  const { toast } = useToast();

  const [persistOverlay, setPersistOverlay] = useState(false);

  const handleCopyUrlToClipBoard = useCallback(
    (url: string) => {
      if (!url) return;
      copyToClipboard(url);
      toast({
        title: t("Copied to clipboard"),
        description: t("asset's URL copied to clipboard"),
      });
    },
    [t, toast],
  );

  const getFileName = (assetUrl: string) => {
    try {
      return assetUrl.split("/").pop() || assetUrl;
    } catch {
      return assetUrl;
    }
  };

  const getItemAssets = () => {
    const assets: { key: string; value: string; field: any }[] = [];
    Object.entries(cmsItem.fields).forEach(([key, value]) => {
      const fieldSchema = cmsModel.schema.fields.find((f) => f.key === key);
      if (
        (fieldSchema?.type === "asset" || fieldSchema?.type === "url") &&
        value
      ) {
        assets.push({ key, value, field: fieldSchema });
      }
    });
    return assets;
  };

  const assets = getItemAssets();

  return (
    <div className="flex h-[600px] flex-col gap-4">
      <ScrollArea className="flex-1">
        <div className="grid min-w-0 grid-cols-5 gap-2 pb-2">
          {assets.map((asset) => (
            <Card
              onDoubleClick={() => onAssetSelect(asset.value)}
              className="group relative cursor-pointer border-transparent bg-secondary hover:border-border">
              <CardContent className="flex items-start justify-center p-2">
                <FileIcon
                  weight="thin"
                  size={70}
                  className="group:hover:opacity-90 opacity-50"
                />
              </CardContent>
              <CardHeader className="px-1 py-0.5">
                <CardTitle className="truncate text-xs dark:font-extralight">
                  {getFileName(asset.value)}
                </CardTitle>
              </CardHeader>
              <CardFooter className="flex flex-col items-start px-1 pb-0.5">
                <p className="text-xs text-zinc-400 dark:font-thin">
                  {asset.field.name}
                </p>
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
                        className="justify-between gap-2"
                        disabled={!asset.value}
                        onClick={() => handleCopyUrlToClipBoard(asset.value)}>
                        {t("Copy Asset URL")}
                        <ClipboardTextIcon weight="light" />
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </div>
            </Card>
          ))}
        </div>
      </ScrollArea>
    </div>
  );
};

export default CmsAssetSelector;
