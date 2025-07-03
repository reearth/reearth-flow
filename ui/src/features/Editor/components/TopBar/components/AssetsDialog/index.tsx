import {
  HardDriveIcon,
  ListIcon,
  UploadSimpleIcon,
} from "@phosphor-icons/react";
import { SquaresFourIcon } from "@phosphor-icons/react/dist/ssr";

import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  Button,
  IconButton,
} from "@flow/components";
import { ALLOWED_ASSET_IMPORT_EXTENSIONS } from "@flow/global-constants";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

import { AssetDeletionDialog } from "./AssetDeletionDialog";
import { AssetsGridView } from "./AssetsGridView";
import { AssetsListView } from "./AssetsListView";
import useHooks from "./hooks";

type Props = {
  onDialogClose: () => void;
};

const AssetsDialog: React.FC<Props> = ({ onDialogClose }) => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();

  const {
    assets,
    isFetching,
    fileInputRef,
    assetToBeDeleted,
    currentPage,
    totalPages,
    currentSortValue,
    sortOptions,
    searchTerm,
    layoutView,
    setAssetToBeDeleted,
    setCurrentPage,
    setSearchTerm,
    handleAssetUploadClick,
    handleAssetCreate,
    handleAssetDelete,
    handleSortChange,
    handleGridView,
    handleListView,
    handleCopyUrlToClipBoard,
    handleAssetDownload,
  } = useHooks({ workspaceId: currentWorkspace?.id ?? "" });

  return (
    <Dialog open={true} onOpenChange={onDialogClose}>
      <DialogContent className="h-[80vh] w-full max-w-4xl overflow-hidden">
        <DialogTitle className="flex items-center font-thin">
          <HardDriveIcon size={24} className="mr-2 inline-block font-thin" />
          {t("Assets")}
        </DialogTitle>
        <DialogContentWrapper>
          <div className="mb-3 flex items-center justify-between overflow-auto">
            <Button
              className="flex gap-2"
              variant="default"
              onClick={handleAssetUploadClick}>
              <UploadSimpleIcon weight="thin" />
              <p className="text-xs dark:font-light">{t("Upload Asset")}</p>
            </Button>

            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2">
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
            </div>
          </div>

          <DialogContentSection className="flex max-h-[500px] flex-col overflow-hidden">
            {layoutView === "grid" ? (
              <AssetsGridView
                assets={assets}
                isFetching={isFetching}
                currentPage={currentPage}
                totalPages={totalPages}
                sortOptions={sortOptions}
                currentSortValue={currentSortValue}
                searchTerm={searchTerm}
                setAssetToBeDeleted={setAssetToBeDeleted}
                setCurrentPage={setCurrentPage}
                setSearchTerm={setSearchTerm}
                onSortChange={handleSortChange}
                onCopyUrlToClipBoard={handleCopyUrlToClipBoard}
                onAssetDownload={handleAssetDownload}
              />
            ) : (
              <AssetsListView
                assets={assets}
                isFetching={isFetching}
                currentPage={currentPage}
                totalPages={totalPages}
                sortOptions={sortOptions}
                currentSortValue={currentSortValue}
                searchTerm={searchTerm}
                setAssetToBeDeleted={setAssetToBeDeleted}
                setCurrentPage={setCurrentPage}
                setSearchTerm={setSearchTerm}
                onSortChange={handleSortChange}
                onCopyUrlToClipBoard={handleCopyUrlToClipBoard}
                onAssetDownload={handleAssetDownload}
              />
            )}
          </DialogContentSection>
        </DialogContentWrapper>
      </DialogContent>
      <AssetDeletionDialog
        assetToBeDeleted={assetToBeDeleted}
        setAssetToBeDeleted={setAssetToBeDeleted}
        onDeleteAsset={handleAssetDelete}
      />
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
