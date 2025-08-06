import {
  CopyIcon,
  DownloadIcon,
  PencilIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";

import { FlowLogo, IconButton, LoadingSkeleton } from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { DataTable as Table } from "@flow/components/DataTable";
import { ASSET_FETCH_RATE } from "@flow/lib/gql/assets/useQueries";
import { useT } from "@flow/lib/i18n";
import type { Asset } from "@flow/types";

type Props = {
  assets?: Asset[];
  isFetching: boolean;
  currentPage: number;
  totalPages: number;
  setCurrentPage?: (page: number) => void;
  setAssetToBeDeleted: (asset: string | undefined) => void;
  setAssetToBeEdited: (asset: Asset | undefined) => void;
  setSearchTerm: (term: string) => void;
  onCopyUrlToClipBoard: (url: string) => void;
  onAssetDownload: (
    e: React.MouseEvent<HTMLAnchorElement>,
    asset: Asset,
  ) => void;
  onAssetDoubleClick?: (asset: Asset) => void;
};
const AssetsListView: React.FC<Props> = ({
  assets,
  currentPage,
  totalPages,
  isFetching,
  setCurrentPage,
  setAssetToBeDeleted,
  setAssetToBeEdited,
  onCopyUrlToClipBoard,
  onAssetDownload,
  onAssetDoubleClick,
}) => {
  const t = useT();

  const resultsPerPage = ASSET_FETCH_RATE;
  const columns: ColumnDef<Asset>[] = [
    {
      accessorKey: "name",
      header: t("Name"),
    },
    {
      accessorKey: "createdAt",
      header: t("Uploaded At"),
    },
    {
      accessorKey: "size",
      header: t("Size"),
    },
    {
      accessorKey: "quickActions",
      header: t("Quick Actions"),
      cell: (row) => (
        <div className="flex gap-1">
          <IconButton
            icon={<PencilIcon />}
            onClick={() => setAssetToBeEdited(row.row.original)}
          />
          <IconButton
            icon={<CopyIcon />}
            onClick={(e) => {
              e.stopPropagation();
              onCopyUrlToClipBoard(row.row.original.url);
            }}
          />
          <a
            href={row.row.original.url}
            onClick={(e) => onAssetDownload(e, row.row.original)}>
            <IconButton icon={<DownloadIcon />} />
          </a>

          <IconButton
            icon={<TrashIcon />}
            onClick={() => setAssetToBeDeleted(row.row.original.id)}
          />
        </div>
      ),
    },
  ];
  return (
    <div className="h-full flex-col overflow-hidden">
      {isFetching ? (
        <LoadingSkeleton className="flex h-full justify-center" />
      ) : assets && assets.length > 0 ? (
        <Table
          columns={columns}
          data={assets}
          showOrdering={false}
          currentPage={currentPage}
          setCurrentPage={setCurrentPage}
          totalPages={totalPages}
          resultsPerPage={resultsPerPage}
          onRowDoubleClick={onAssetDoubleClick}
        />
      ) : (
        <div className="flex h-full items-center justify-center">
          <BasicBoiler
            text={t("No Assets")}
            icon={<FlowLogo className=" mb-3 size-16 text-accent" />}
          />
        </div>
      )}
    </div>
  );
};

export { AssetsListView };
