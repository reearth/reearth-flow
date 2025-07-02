import { ListIcon, PlusIcon } from "@phosphor-icons/react";
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
  // setShowDialog: (show: boolean) => void;
};

const AssetsDialog: React.FC<Props> = ({ setShowDialog }) => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();

  const {
    assets,
    assetToBeDeleted,
    fileInputRefProject,
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
  } = useHooks({ workspaceId: currentWorkspace?.id ?? "" });

  return (
    <Dialog open={true}>
      <DialogContent className="h-[80vh] w-full max-w-4xl overflow-hidden">
        <DialogTitle>{t("Assets")}</DialogTitle>
        <DialogContentWrapper>
          <div className="mb-3 flex items-center justify-between overflow-auto">
            <Button
              className="flex gap-2"
              variant="default"
              onClick={handleAssetUploadClick}>
              <PlusIcon weight="thin" />
              <p className="text-xs dark:font-light">{t("New Asset")}</p>
            </Button>

            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2">
                <IconButton
                  size="icon"
                  variant="outline"
                  className={layoutView === "grid" ? "bg-accent" : ""}
                  tooltipText={t("Grid Layout")}
                  onClick={handleGridView}
                  icon={<SquaresFourIcon />}
                />
                <IconButton
                  size="icon"
                  variant="outline"
                  className={layoutView === "list" ? "bg-accent" : ""}
                  tooltipText={t("List Layout")}
                  onClick={handleListView}
                  icon={<ListIcon />}
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
                handleOrderChange={handleOrderChange}
                setAssetToBeDeleted={setAssetToBeDeleted}
                setCurrentPage={setCurrentPage}
                searchTerm={searchTerm}
                setSearchTerm={setSearchTerm}
              />
            ) : (
              <AssetsListView
                assets={assets}
                isFetching={isFetching}
                currentPage={currentPage}
                totalPages={totalPages}
                setAssetToBeDeleted={setAssetToBeDeleted}
                setCurrentPage={setCurrentPage}
                sortOptions={sortOptions}
                currentSortValue={currentSortValue}
                handleSortChange={handleOrderChange}
                searchTerm={searchTerm}
                setSearchTerm={setSearchTerm}
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
        ref={fileInputRefProject}
        onChange={handleAssetCreate}
        style={{ display: "none" }}
      />
    </Dialog>
  );
};

export default AssetsDialog;
