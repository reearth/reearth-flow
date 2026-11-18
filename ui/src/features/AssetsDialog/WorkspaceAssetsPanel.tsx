import { FileArrowUpIcon, ListIcon } from "@phosphor-icons/react";
import { SquaresFourIcon } from "@phosphor-icons/react/dist/ssr";

import {
  Button,
  DialogContentSection,
  IconButton,
  Pagination,
  Input,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  LoadingSkeleton,
} from "@flow/components";
import { ALLOWED_ASSET_IMPORT_EXTENSIONS } from "@flow/global-constants";
import { useAssets } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import type { Asset } from "@flow/types";

import {
  AssetDeletionDialog,
  AssetEditDialog,
  AssetsGridView,
  AssetsListView,
} from "./components";

type Props = {
  workspaceId: string;
  className?: string;
  selectedAssetId?: string;
  onAssetClick?: (asset: Asset) => void;
  onAssetDoubleClick?: (asset: Asset) => void;
  onAssetDelete?: (id: string) => void;
};

const WorkspaceAssetsPanel: React.FC<Props> = ({
  workspaceId,
  className,
  selectedAssetId,
  onAssetClick,
  onAssetDoubleClick,
  onAssetDelete,
}) => {
  const t = useT();

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
    uploadingTitle,
    uploadPercent,
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

  const handleAssetDoubleClick = onAssetDoubleClick ?? setAssetToBeEdited;

  return (
    <>
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="flex items-center gap-2 py-3">
            <Input
              placeholder={t("Search") + "..."}
              value={searchTerm ?? ""}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="h-[36px] max-w-sm"
            />
            <Select
              value={currentSortValue}
              onValueChange={(v) => v != null && handleSortChange(v)}
              items={sortOptions}>
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
            className={layoutView === "grid" ? "bg-accent" : ""}
            tooltipText={t("Grid Layout")}
            onClick={handleGridView}
            icon={<SquaresFourIcon size={"18px"} />}
          />
          <IconButton
            size="icon"
            variant="outline"
            className={layoutView === "list" ? "bg-accent" : ""}
            tooltipText={t("List Layout")}
            onClick={handleListView}
            icon={<ListIcon size={"18px"} />}
          />
        </div>
        <Button
          variant="default"
          onClick={handleAssetUploadClick}
          disabled={isCreatingAsset}>
          <FileArrowUpIcon weight="thin" />
          <p className="text-xs dark:font-light">{t("Upload")}</p>
        </Button>
      </div>

      <DialogContentSection
        className={`flex flex-col overflow-hidden ${className ?? "h-[500px]"}`}>
        {!isCreatingAsset ? (
          <>
            {layoutView === "list" ? (
              <AssetsListView
                assets={assets}
                selectedAssetId={selectedAssetId}
                isFetching={isFetching}
                isDebouncingSearch={isDebouncingSearch}
                isDeleting={isDeleting}
                currentPage={currentPage}
                totalPages={totalPages}
                setAssetToBeDeleted={setAssetToBeDeleted}
                setAssetToBeEdited={setAssetToBeEdited}
                setCurrentPage={setCurrentPage}
                setSearchTerm={setSearchTerm}
                onCopyUrlToClipBoard={handleCopyUrlToClipBoard}
                onAssetDownload={handleAssetDownload}
                onAssetClick={onAssetClick}
                onAssetDoubleClick={handleAssetDoubleClick}
              />
            ) : (
              <AssetsGridView
                assets={assets}
                selectedAssetId={selectedAssetId}
                isFetching={isFetching}
                isDebouncingSearch={isDebouncingSearch}
                isDeleting={isDeleting}
                setAssetToBeDeleted={setAssetToBeDeleted}
                setAssetToBeEdited={setAssetToBeEdited}
                onCopyUrlToClipBoard={handleCopyUrlToClipBoard}
                onAssetDownload={handleAssetDownload}
                onAssetClick={onAssetClick}
                onAssetDoubleClick={handleAssetDoubleClick}
              />
            )}
          </>
        ) : (
          <div className="h-full">
            <LoadingSkeleton title={uploadingTitle} progress={uploadPercent} />
          </div>
        )}
        {assets && assets.length > 0 && (
          <div className="mb-3">
            <Pagination
              currentPage={currentPage}
              setCurrentPage={setCurrentPage}
              totalPages={totalPages}
            />
          </div>
        )}
      </DialogContentSection>
      <AssetDeletionDialog
        assetToBeDeleted={assetToBeDeleted}
        setAssetToBeDeleted={setAssetToBeDeleted}
        onDeleteAsset={(id) => {
          onAssetDelete?.(id);
          handleAssetDelete(id);
        }}
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
    </>
  );
};

export default WorkspaceAssetsPanel;
