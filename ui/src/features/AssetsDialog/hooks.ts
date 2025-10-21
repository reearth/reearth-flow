import { ChangeEvent, useCallback, useEffect, useRef, useState } from "react";

import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useDebouncedSearch } from "@flow/hooks";
import { useAsset } from "@flow/lib/gql/assets";
import { useT } from "@flow/lib/i18n";
import { Asset, AssetOrderBy } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";
import { copyToClipboard } from "@flow/utils/copyToClipboard";

export default ({
  workspaceId,
  onDialogClose,
  onAssetDoubleClick,
}: {
  workspaceId: string;
  onDialogClose: () => void;
  onAssetDoubleClick?: (asset: Asset) => void;
}) => {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const t = useT();
  const { toast } = useToast();
  const { useGetAssets, createAsset, updateAsset, deleteAsset } = useAsset();
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [currentOrderBy, setCurrentOrderBy] = useState<AssetOrderBy>(
    AssetOrderBy.CreatedAt,
  );
  const [currentOrderDir, setCurrentOrder] = useState<OrderDirection>(
    OrderDirection.Desc,
  );

  const { searchTerm, isDebouncingSearch, setSearchTerm } = useDebouncedSearch({
    initialSearchTerm: "",
    delay: 300,
    onDebounced: () => {
      refetch();
    },
  });

  const [isDeleting, setIsDeleting] = useState<boolean>(false);

  const [assetToBeEdited, setAssetToBeEdited] = useState<Asset | undefined>(
    undefined,
  );
  const [assetToBeDeleted, setAssetToBeDeleted] = useState<string | undefined>(
    undefined,
  );

  const [layoutView, setLayoutView] = useState<"list" | "grid">("list");

  const { page, refetch, isFetching } = useGetAssets(workspaceId, searchTerm, {
    page: currentPage,
    orderDir: currentOrderDir,
    orderBy: currentOrderBy,
  });

  const totalPages = page?.totalPages as number;

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
      value: `${AssetOrderBy.Size}_${OrderDirection.Desc}`,
      label: t("Size Small to Large"),
    },
    {
      value: `${AssetOrderBy.Size}_${OrderDirection.Asc}`,
      label: t("Size Large to Small"),
    },
  ];

  const currentSortValue = `${currentOrderBy}_${currentOrderDir}`;

  useEffect(() => {
    (async () => {
      await refetch();
    })();
  }, [currentPage, currentOrderDir, currentOrderBy, refetch]);

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
      createAsset(file, workspaceId);
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

  const handleSortChange = useCallback((newSortValue: string) => {
    const [orderBy, orderDir] = newSortValue.split("_") as [
      AssetOrderBy,
      OrderDirection,
    ];
    setCurrentOrderBy(orderBy);
    setCurrentOrder(orderDir);
  }, []);

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

        const fileName = `${asset.name}.${asset.url.split("/").pop()?.split(".").pop()}`;

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
    [],
  );

  const handleAssetDoubleClick = useCallback(
    (asset: Asset) => {
      if (onAssetDoubleClick) {
        onAssetDoubleClick(asset);
        onDialogClose();
      } else {
        setAssetToBeEdited(asset);
      }
    },
    [onAssetDoubleClick, setAssetToBeEdited, onDialogClose],
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
    handleAssetDoubleClick,
  };
};
