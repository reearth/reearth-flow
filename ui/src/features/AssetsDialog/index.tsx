import { FileArrowUpIcon, FileIcon, ListIcon } from "@phosphor-icons/react";
import { SquaresFourIcon } from "@phosphor-icons/react/dist/ssr";

import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  Button,
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
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { Asset } from "@flow/types";

import {
  AssetDeletionDialog,
  AssetEditDialog,
  AssetsGridView,
  AssetsListView,
} from "./components";
import useHooks from "./hooks";

type Props = {
  onDialogClose: () => void;
  onAssetDoubleClick?: (asset: Asset) => void;
};

const AssetsDialog: React.FC<Props> = ({
  onDialogClose,
  onAssetDoubleClick,
}) => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();

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
    handleAssetDoubleClick,
  } = useHooks({
    workspaceId: currentWorkspace?.id ?? "",
    onDialogClose,
    onAssetDoubleClick,
  });

  return (
    <Dialog open onOpenChange={onDialogClose}>
      <DialogContent className="max-h-[800px] w-full max-w-4xl overflow-hidden">
        <DialogTitle className="flex items-center gap-2">
          <FileIcon />
          {t("Workspace Assets")}
        </DialogTitle>
        <DialogContentWrapper>
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
                  onValueChange={handleSortChange}>
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

            <Button
              variant="default"
              onClick={handleAssetUploadClick}
              disabled={isCreatingAsset}>
              <FileArrowUpIcon weight="thin" />
              <p className="text-xs dark:font-light">{t("Upload")}</p>
            </Button>
          </div>

          <DialogContentSection className="flex h-[500px] flex-col overflow-hidden">
            {!isCreatingAsset ? (
              <>
                {layoutView === "list" ? (
                  <AssetsListView
                    assets={assets}
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
                    onAssetDoubleClick={handleAssetDoubleClick}
                  />
                ) : (
                  <AssetsGridView
                    assets={assets}
                    isFetching={isFetching}
                    isDebouncingSearch={isDebouncingSearch}
                    isDeleting={isDeleting}
                    setAssetToBeDeleted={setAssetToBeDeleted}
                    setAssetToBeEdited={setAssetToBeEdited}
                    onCopyUrlToClipBoard={handleCopyUrlToClipBoard}
                    onAssetDownload={handleAssetDownload}
                    onAssetDoubleClick={handleAssetDoubleClick}
                  />
                )}
              </>
            ) : (
              <div className="h-full">
                <LoadingSkeleton title={t("Uploading Asset...")} />
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
        </DialogContentWrapper>
      </DialogContent>
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
    </Dialog>
  );
};

export default AssetsDialog;
