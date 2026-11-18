import { ListIcon, WarningIcon } from "@phosphor-icons/react";
import { SquaresFourIcon } from "@phosphor-icons/react/dist/ssr";

import {
  Button,
  DialogContentSection,
  FlowLogo,
  Icon,
  IconButton,
  Input,
  LoadingSkeleton,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { DataTable as Table } from "@flow/components/DataTable";
import { getIconFileType } from "@flow/features/AssetsDialog/components/utils";
import { useT } from "@flow/lib/i18n";
import type { AppColumnDef } from "@flow/lib/table/features";
import type { CmsAsset } from "@flow/types/cmsIntegration";
import { formatDate, formatFileSize } from "@flow/utils";

import type { CmsAssetFormat, CmsAssetSort } from "../../utils";
import { extensionOf } from "../../utils";

import CmsAssetCard from "./CmsAssetCard";
import useHooks from "./hooks";

type Props = {
  workspaceId: string;
  selectedAssetId?: string;
  onAssetClick: (asset: CmsAsset) => void;
  onAssetDoubleClick: (asset: CmsAsset) => void;
};

const CmsAssetsPanel: React.FC<Props> = ({
  workspaceId,
  selectedAssetId,
  onAssetClick,
  onAssetDoubleClick,
}) => {
  const t = useT();
  const {
    assets,
    projectNames,
    hasProjects,
    isLoading,
    isFetchingNextPage,
    hasNextPage,
    failedProjectCount,
    searchTerm,
    format,
    projectId,
    sort,
    layoutView,
    formatOptions,
    projectOptions,
    sortOptions,
    formatLabelOf,
    setSearchTerm,
    setFormat,
    setProjectId,
    setSort,
    setLayoutView,
    fetchNextPage,
  } = useHooks({ workspaceId });

  const columns: AppColumnDef<CmsAsset>[] = [
    {
      accessorKey: "filename",
      header: t("Name"),
      cell: ({ row }) => {
        const icon = getIconFileType(extensionOf(row.original.filename));
        return (
          <div className="flex items-center gap-1.5">
            {icon && <Icon icon={icon} size={25} className="opacity-50" />}
            <strong className="truncate">{row.original.filename}</strong>
          </div>
        );
      },
    },
    {
      accessorKey: "projectId",
      header: t("Project"),
      cell: ({ row }) => projectNames.get(row.original.projectId) ?? "-",
    },
    {
      accessorKey: "format",
      header: t("Format"),
      cell: ({ row }) => formatLabelOf(row.original.filename),
    },
    {
      accessorKey: "createdAt",
      header: t("Uploaded At"),
      cell: ({ row }) => formatDate(row.original.createdAt),
    },
    {
      accessorKey: "size",
      header: t("Size"),
      cell: ({ row }) => formatFileSize(row.original.size),
    },
  ];

  return (
    <>
      <div className="flex flex-wrap items-center gap-2 py-3">
        <Input
          placeholder={t("Search") + "..."}
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="h-[36px] w-[200px]"
        />
        <Select
          value={format}
          onValueChange={(v) => v != null && setFormat(v as CmsAssetFormat)}
          items={formatOptions}>
          <SelectTrigger className="h-[36px] w-[150px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {formatOptions.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select
          value={projectId}
          onValueChange={(v) => v != null && setProjectId(v)}
          items={projectOptions}>
          <SelectTrigger className="h-[36px] w-[180px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {projectOptions.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select
          value={sort}
          onValueChange={(v) => v != null && setSort(v as CmsAssetSort)}
          items={sortOptions}>
          <SelectTrigger className="h-[36px] w-[170px]">
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
        <IconButton
          size="icon"
          variant="outline"
          className={layoutView === "grid" ? "bg-accent" : ""}
          tooltipText={t("Grid Layout")}
          onClick={() => setLayoutView("grid")}
          icon={<SquaresFourIcon size={"18px"} />}
        />
        <IconButton
          size="icon"
          variant="outline"
          className={layoutView === "list" ? "bg-accent" : ""}
          tooltipText={t("List Layout")}
          onClick={() => setLayoutView("list")}
          icon={<ListIcon size={"18px"} />}
        />
      </div>
      {failedProjectCount > 0 && (
        <div className="flex items-center gap-2 text-xs text-warning">
          <WarningIcon />
          {t(
            "Assets from {{total}} project(s) could not be loaded. Results may be incomplete.",
            { total: failedProjectCount },
          )}
        </div>
      )}
      <DialogContentSection className="flex h-[500px] flex-col overflow-hidden">
        <div className="flex-1 overflow-y-auto">
          {isLoading ? (
            <LoadingSkeleton className="flex h-full justify-center" />
          ) : assets.length === 0 ? (
            <div className="flex h-full items-center justify-center">
              <BasicBoiler
                text={hasProjects ? t("No Assets") : t("No Projects Found")}
                icon={<FlowLogo className="mb-3 size-16 text-accent" />}
              />
            </div>
          ) : layoutView === "grid" ? (
            <div className="grid min-w-0 grid-cols-5 gap-2 p-2">
              {assets.map((asset) => (
                <CmsAssetCard
                  key={asset.id}
                  asset={asset}
                  projectName={projectNames.get(asset.projectId)}
                  formatLabel={formatLabelOf(asset.filename)}
                  selected={asset.id === selectedAssetId}
                  onClick={onAssetClick}
                  onDoubleClick={onAssetDoubleClick}
                />
              ))}
            </div>
          ) : (
            <Table
              columns={columns}
              data={assets}
              showOrdering={false}
              onRowClick={onAssetClick}
              isRowSelected={(a) => a.id === selectedAssetId}
              onRowDoubleClick={onAssetDoubleClick}
            />
          )}
        </div>
        {hasNextPage && !isLoading && (
          <div className="flex justify-center py-2">
            <Button
              variant="outline"
              size="sm"
              disabled={isFetchingNextPage}
              onClick={() => fetchNextPage()}>
              {isFetchingNextPage ? t("Loading...") : t("Load more")}
            </Button>
          </div>
        )}
      </DialogContentSection>
    </>
  );
};

export default CmsAssetsPanel;
