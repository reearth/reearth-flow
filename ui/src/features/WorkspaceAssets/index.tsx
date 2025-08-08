import {
  ListIcon,
  SquaresFourIcon,
  UploadSimpleIcon,
} from "@phosphor-icons/react";

import {
  Button,
  IconButton,
  Input,
  Pagination,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { ALLOWED_ASSET_IMPORT_EXTENSIONS } from "@flow/global-constants";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

import {
  AssetDeletionDialog,
  AssetEditDialog,
  AssetsGridView,
  AssetsListView,
} from "../AssetsDialog/components";

import useHooks from "./hooks";

const AssetsManager: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const {
    assets,
    isDebouncing,
    isFetching,
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
  } = useHooks({
    workspaceId: currentWorkspace?.id ?? "",
  });

  return (
    <div className="flex h-full flex-1 flex-col px-6 pt-4 pb-2">
      <div className="flex h-[50px] flex-shrink-0 items-center justify-between gap-2 border-b pb-4">
        <p className="text-lg dark:font-extralight">{t("Assets")}</p>
        <Button
          className="flex gap-2"
          variant="default"
          onClick={handleAssetUploadClick}>
          <UploadSimpleIcon weight="thin" />
          <p className="text-xs dark:font-light">{t("Upload Asset")}</p>
        </Button>
      </div>
      <div className="mt-4 flex min-h-0 w-full flex-1 flex-col gap-4">
        <div className="flex flex-shrink-0 items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="flex items-center gap-2 py-3">
              <Input
                placeholder={t("Search") + "..."}
                value={searchTerm ?? ""}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="h-[36px] max-w-sm"
              />
              <Select value={currentSortValue} onValueChange={handleSortChange}>
                <SelectTrigger className="h-[36px] w-[150px]">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {sortOptions.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <IconButton
              size="icon"
              variant="outline"
              className={layoutView === "list" ? "bg-accent" : ""}
              tooltipText={t("List Layout")}
              onClick={handleListView}
              icon={<ListIcon size={"18px"} />}
            />
            <IconButton
              size="icon"
              variant="outline"
              className={layoutView === "grid" ? "bg-accent" : ""}
              tooltipText={t("Grid Layout")}
              onClick={handleGridView}
              icon={<SquaresFourIcon size={"18px"} />}
            />
          </div>
        </div>
        <div className="flex min-h-0 flex-1 flex-col">
          {layoutView === "list" ? (
            <AssetsListView
              assets={assets}
              isDebouncing={isDebouncing}
              isFetching={isFetching}
              isDeleting={isDeleting}
              currentPage={currentPage}
              totalPages={totalPages}
              setAssetToBeDeleted={setAssetToBeDeleted}
              setAssetToBeEdited={setAssetToBeEdited}
              setCurrentPage={setCurrentPage}
              setSearchTerm={setSearchTerm}
              onCopyUrlToClipBoard={handleCopyUrlToClipBoard}
              onAssetDownload={handleAssetDownload}
              onAssetDoubleClick={handleAssetDoubleClick}
            />
          ) : (
            <AssetsGridView
              assets={assets}
              isDebouncing={isDebouncing}
              isFetching={isFetching}
              isDeleting={isDeleting}
              setAssetToBeDeleted={setAssetToBeDeleted}
              setAssetToBeEdited={setAssetToBeEdited}
              onCopyUrlToClipBoard={handleCopyUrlToClipBoard}
              onAssetDownload={handleAssetDownload}
              onAssetDoubleClick={handleAssetDoubleClick}
            />
          )}
        </div>

        {assets && assets.length > 0 && (
          <div className="mt-4 flex-shrink-0">
            <Pagination
              currentPage={currentPage}
              setCurrentPage={setCurrentPage}
              totalPages={totalPages}
            />
          </div>
        )}
      </div>
      <AssetDeletionDialog
        assetToBeDeleted={assetToBeDeleted}
        setAssetToBeDeleted={setAssetToBeDeleted}
        onDeleteAsset={handleAssetDelete}
      />
      {assetToBeEdited && (
        <AssetEditDialog
          assetToBeEdited={assetToBeEdited}
          setAssetToBeEdited={setAssetToBeEdited}
          onUpdateAsset={handleAssetUpdate}
        />
      )}
      <input
        type="file"
        accept={ALLOWED_ASSET_IMPORT_EXTENSIONS}
        ref={fileInputRef}
        onChange={handleAssetCreate}
        style={{ display: "none" }}
      />
    </div>
  );
};

export { AssetsManager };
