import { PlusIcon } from "@phosphor-icons/react";
import { SquaresFourIcon } from "@phosphor-icons/react/dist/ssr";

import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  LoadingSkeleton,
  FlowLogo,
  Pagination,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  Button,
  IconButton,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { ALLOWED_ASSET_IMPORT_EXTENSIONS } from "@flow/global-constants";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

import { Assetcard } from "./AssetCard";
import { AssetDeletionDialog } from "./AssetDeletionDialog";
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
    currentOrder,
    orderDirections,
    handleOrderChange,
    handleAssetUploadClick,
    handleAssetCreate,
    handleAssetDelete,
    setCurrentPage,
  } = useHooks({ workspaceId: currentWorkspace?.id ?? "" });

  return (
    <Dialog open={true}>
      <DialogContent size="2xl">
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
              {currentOrder && (
                <Select
                  value={currentOrder || "DESC"}
                  onValueChange={handleOrderChange}>
                  <SelectTrigger className="w-[100px]">
                    <SelectValue placeholder={orderDirections.ASC} />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(orderDirections).map(([value, label]) => (
                      <SelectItem key={value} value={value}>
                        {label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              )}
              <div className="flex items-center gap-2">
                <IconButton
                  size="icon"
                  variant="outline"
                  tooltipText={t("Grid Layout")}
                  icon={<SquaresFourIcon />}
                />
                <IconButton
                  size="icon"
                  variant="outline"
                  tooltipText={t("List Layout")}
                  icon={<SquaresFourIcon />}
                />
              </div>
            </div>
          </div>

          <DialogContentSection className="flex max-h-[60vh] flex-col overflow-hidden">
            {isFetching ? (
              <LoadingSkeleton />
            ) : assets && assets.length > 0 ? (
              <div className="flex-1 overflow-y-auto">
                <div className="grid min-w-0 grid-cols-1 gap-2 sm:grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
                  {assets.map((a) => (
                    <Assetcard
                      key={a.id}
                      asset={a}
                      setAssetToBeDeleted={setAssetToBeDeleted}
                    />
                  ))}
                </div>
              </div>
            ) : (
              <BasicBoiler
                text={t("No Assets")}
                icon={<FlowLogo className="size-16 text-accent" />}
              />
            )}

            <div className="mb-3">
              <Pagination
                currentPage={currentPage}
                setCurrentPage={setCurrentPage}
                totalPages={totalPages}
              />
            </div>
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
