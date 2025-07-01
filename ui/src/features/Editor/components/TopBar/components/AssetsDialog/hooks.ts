import { ChangeEvent, useCallback, useEffect, useRef, useState } from "react";

import { useAsset } from "@flow/lib/gql/assets";
import { useT } from "@flow/lib/i18n";
import { AssetSortType } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";

export default ({ workspaceId }: { workspaceId: string }) => {
  const fileInputRefProject = useRef<HTMLInputElement>(null);
  const t = useT();

  const { useGetAssets, createAsset, removeAsset } = useAsset();
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [currentOrder, setCurrentOrder] = useState<OrderDirection>(
    OrderDirection.Desc,
  );

  const [assetToBeDeleted, setAssetToBeDeleted] = useState<string | undefined>(
    undefined,
  );
  const [layoutView, setLayoutView] = useState<"grid" | "list">("grid");

  const handleGridView = () => setLayoutView("grid");

  const handleListView = () => setLayoutView("list");

  const { page, refetch, isFetching } = useGetAssets(
    workspaceId,
    AssetSortType.Date,
    {
      page: currentPage,
      orderDir: currentOrder,
      orderBy: "createdAt",
    },
  );
  const totalPages = page?.totalPages as number;

  const assets = page?.assets;
  const orderDirections: Record<OrderDirection, string> = {
    DESC: t("Newest"),
    ASC: t("Oldest"),
  };
  const handleOrderChange = () => {
    setCurrentOrder?.(
      currentOrder === OrderDirection.Asc
        ? OrderDirection.Desc
        : OrderDirection.Asc,
    );
  };

  useEffect(() => {
    (async () => {
      await refetch();
    })();
  }, [currentPage, currentOrder, refetch]);

  const handleAssetUploadClick = useCallback(() => {
    fileInputRefProject.current?.click();
  }, []);

  const handleAssetCreate = useCallback(
    async (e: ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;

      if (!workspaceId) return console.error("Missing current workspace");

      try {
        await createAsset({
          workspaceId,
          file,
        });
      } catch (error) {
        console.error("Failed to upload file:", error);
      }
    },
    [createAsset, workspaceId],
  );

  const handleAssetDelete = async (id: string) => {
    setAssetToBeDeleted(undefined);
    await removeAsset({ assetId: id });
  };

  return {
    assets,
    fileInputRefProject,
    assetToBeDeleted,
    setAssetToBeDeleted,
    currentPage,
    totalPages,
    isFetching,
    currentOrder,
    orderDirections,
    layoutView,
    handleOrderChange,
    handleAssetUploadClick,
    handleAssetCreate,
    handleAssetDelete,
    handleGridView,
    handleListView,
    setCurrentPage,
  };
};
