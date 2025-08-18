import {
  CopyIcon,
  DownloadIcon,
  PencilIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";

import { IconButton, LoadingTableSkeleton } from "@flow/components";
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
  isFetching,
  currentPage,
  totalPages,
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
    <div className="h-full flex-1 overflow-hidden">
      {isFetching ? (
        <LoadingTableSkeleton columns={4} rows={ASSET_FETCH_RATE} />
      ) : (
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
      )}
    </div>
  );
};

export { AssetsListView };
