import { useCallback } from "react";

import { useAssets } from "@flow/hooks";
import { Asset } from "@flow/types";

export default ({
  workspaceId,
  onDialogClose,
  onAssetDoubleClick,
}: {
  workspaceId: string;
  onDialogClose: () => void;
  onAssetDoubleClick?: (asset: Asset) => void;
}) => {
  const {
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
  } = useAssets({ workspaceId });

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
    handleAssetDoubleClick,
  };
};
