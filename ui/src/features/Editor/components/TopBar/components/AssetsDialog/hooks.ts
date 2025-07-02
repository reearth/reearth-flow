import { ChangeEvent, useCallback, useEffect, useRef, useState } from "react";

import { useDebouncedSearch } from "@flow/hooks";
import { useAsset } from "@flow/lib/gql/assets";
import { useT } from "@flow/lib/i18n";
import { AssetOrderBy } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";

export default ({ workspaceId }: { workspaceId: string }) => {
  const fileInputRefProject = useRef<HTMLInputElement>(null);
  const t = useT();

  const { useGetAssets, createAsset, removeAsset } = useAsset();
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [currentOrderBy, setCurrentOrderBy] = useState<AssetOrderBy>(
    AssetOrderBy.CreatedAt,
  );
  const [currentOrderDir, setCurrentOrder] = useState<OrderDirection>(
    OrderDirection.Desc,
  );

  const { searchTerm, setSearchTerm } = useDebouncedSearch({
    initialTerm: "",
    delay: 500,
    onDebounced: () => {
      refetch();
    },
  });
  const [assetToBeDeleted, setAssetToBeDeleted] = useState<string | undefined>(
    undefined,
  );
  const [layoutView, setLayoutView] = useState<"grid" | "list">("grid");

  const handleGridView = () => setLayoutView("grid");

  const handleListView = () => setLayoutView("list");

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

  const handleOrderChange = (newSortValue: string) => {
    const [orderBy, orderDir] = newSortValue.split("_") as [
      AssetOrderBy,
      OrderDirection,
    ];
    setCurrentOrderBy(orderBy);
    setCurrentOrder(orderDir);
  };

  useEffect(() => {
    (async () => {
      await refetch();
    })();
  }, [currentPage, currentOrderDir, currentOrderBy, refetch]);

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
    sortOptions,
    currentSortValue,
    layoutView,
    searchTerm,
    handleOrderChange,
    handleAssetUploadClick,
    handleAssetCreate,
    handleAssetDelete,
    handleGridView,
    handleListView,
    setCurrentPage,
    setSearchTerm,
  };
};
