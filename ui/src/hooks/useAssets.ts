import { ChangeEvent, useCallback, useRef, useState } from "react";

import { useToast } from "@flow/features/NotificationSystem/useToast";
import { ALLOWED_ASSET_IMPORT_EXTENSIONS } from "@flow/global-constants";
import { usePagination } from "@flow/hooks";
import { useAsset } from "@flow/lib/gql/assets";
import { useT } from "@flow/lib/i18n";
import { Asset, AssetOrderBy } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";
import { copyToClipboard } from "@flow/utils/copyToClipboard";

export default ({ workspaceId }: { workspaceId: string }) => {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const t = useT();
  const { toast } = useToast();
  const {
    useGetAssets,
    createAsset,
    updateAsset,
    deleteAsset,
    isCreatingAsset,
  } = useAsset();

  const availableExtensions = ALLOWED_ASSET_IMPORT_EXTENSIONS.split(",").map(
    (ext) => ext.trim(),
  );

  const {
    page,
    totalPages,
    isFetching,
    currentPage,
    currentSortValue,
    searchTerm,
    isDebouncingSearch,
    setCurrentPage,
    setSearchTerm,
    handleSortChange,
  } = usePagination({
    useDataQuery: useGetAssets,
    workspaceId,
    defaultOrderBy: AssetOrderBy.CreatedAt,
  });

  const [isDeleting, setIsDeleting] = useState<boolean>(false);

  const [assetToBeEdited, setAssetToBeEdited] = useState<Asset | undefined>(
    undefined,
  );
  const [assetToBeDeleted, setAssetToBeDeleted] = useState<string | undefined>(
    undefined,
  );

  const [layoutView, setLayoutView] = useState<"list" | "grid">("grid");

  const assets = page?.assets;
  const sortOptions = [
    {
      value: `${AssetOrderBy.CreatedAt}_${OrderDirection.Desc}`,
      label: t("Last Uploaded"),
    },
    {
      value: `${AssetOrderBy.CreatedAt}_${OrderDirection.Asc}`,
      label: t("First Uploaded"),
    },
    { value: `${AssetOrderBy.Name}_${OrderDirection.Asc}`, label: t("A To Z") },
    {
      value: `${AssetOrderBy.Name}_${OrderDirection.Desc}`,
      label: t("Z To A"),
    },
    {
      value: `${AssetOrderBy.Size}_${OrderDirection.Asc}`,
      label: t("Size Small to Large"),
    },
    {
      value: `${AssetOrderBy.Size}_${OrderDirection.Desc}`,
      label: t("Size Large to Small"),
    },
  ];

  const handleGridView = () => setLayoutView("grid");

  const handleListView = () => setLayoutView("list");

  const handleAssetUploadClick = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleAssetCreate = useCallback(
    async (e: ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;
      if (!workspaceId) return console.error("Missing current workspace");
      createAsset(workspaceId, file);
    },
    [createAsset, workspaceId],
  );

  const handleAssetUpdate = useCallback(
    async (updatedName: string) => {
      if (!assetToBeEdited) return;

      await updateAsset({
        assetId: assetToBeEdited.id,
        name: updatedName,
      });

      setAssetToBeEdited(undefined);
    },
    [assetToBeEdited, updateAsset],
  );

  const handleAssetDelete = async (id: string) => {
    try {
      setIsDeleting(true);
      setAssetToBeDeleted(undefined);
      await deleteAsset({ assetId: id });
    } catch (error) {
      console.error("Failed to delete asset:", error);
    } finally {
      setIsDeleting(false);
    }
  };

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

  const handleAssetDownload = useCallback(
    async (e: React.MouseEvent<HTMLAnchorElement>, asset: Asset) => {
      e.preventDefault();
      try {
        const response = await fetch(asset.url);
        if (!response.ok) throw new Error("Failed to fetch");

        const blob = await response.blob();
        const blobUrl = window.URL.createObjectURL(blob);

        const link = document.createElement("a");
        link.href = blobUrl;
        let fileName;
        if (
          availableExtensions.some((ext: string) => asset.name.endsWith(ext))
        ) {
          fileName = asset.name;
        } else {
          const extension = asset.url.split("/").pop()?.split(".").pop();
          fileName = extension ? `${asset.name}.${extension}` : asset.name;
        }
        link.download = fileName.replace(/"/g, "");
        document.body.appendChild(link);
        link.click();

        document.body.removeChild(link);
        window.URL.revokeObjectURL(blobUrl);
      } catch (error) {
        console.error("Download failed:", error);
        window.location.href = asset.url;
      }
    },
    [availableExtensions],
  );

  return {
    assets,
    isFetching,
    isDebouncingSearch,
    isDeleting,
    fileInputRef,
    assetToBeDeleted,
    assetToBeEdited,
    currentPage,
    totalPages,
    currentSortValue,
    sortOptions,
    searchTerm,
    layoutView,
    isCreatingAsset,
    setAssetToBeDeleted,
    setAssetToBeEdited,
    setCurrentPage,
    setSearchTerm,
    handleAssetUploadClick,
    handleAssetCreate,
    handleAssetUpdate,
    handleAssetDelete,
    handleSortChange,
    handleGridView,
    handleListView,
    handleCopyUrlToClipBoard,
    handleAssetDownload,
  };
};
