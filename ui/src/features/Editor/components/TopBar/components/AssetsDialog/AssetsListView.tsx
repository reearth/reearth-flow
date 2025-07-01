import { PencilLineIcon, TrashIcon } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";

import { ButtonWithTooltip } from "@flow/components";
import { DataTable as Table } from "@flow/components/DataTable";
import { DEPLOYMENT_FETCH_RATE } from "@flow/lib/gql/deployment/useQueries";
import { useT } from "@flow/lib/i18n";
import type { Asset } from "@flow/types";

type Props = {
  assets: Asset[];
  currentPage: number;
  totalPages: number;
  sortOptions: { value: string; label: string }[];
  currentSortValue: string;
  handleSortChange: (value: string) => void;
  setCurrentPage?: (page: number) => void;
  setAssetToBeDeleted: (asset: string | undefined) => void;
};
const AssetsListView: React.FC<Props> = ({
  assets,
  currentPage,
  totalPages,
  sortOptions,
  currentSortValue,
  handleSortChange,
  setCurrentPage,
  setAssetToBeDeleted,
}) => {
  const t = useT();

  const resultsPerPage = DEPLOYMENT_FETCH_RATE;
  const columns: ColumnDef<Asset>[] = [
    {
      accessorKey: "name",
      header: t("Name"),
    },
    {
      accessorKey: "createdAt",
      header: t("Created At"),
    },
    {
      accessorKey: "size",
      header: t("Size"),
    },
    {
      accessorKey: "url",
      header: t("Path"),
    },
    {
      accessorKey: "quickActions",
      header: t("Quick Actions"),
      cell: (row) => (
        <div className="flex gap-2">
          <ButtonWithTooltip
            variant="outline"
            size="icon"
            tooltipText={t("Edit Asset")}>
            <PencilLineIcon />
          </ButtonWithTooltip>
          <ButtonWithTooltip
            variant="destructive"
            size="icon"
            tooltipText={t("Delete Asset")}
            onClick={() => setAssetToBeDeleted(row.row.original.id)}>
            <TrashIcon />
          </ButtonWithTooltip>
        </div>
      ),
    },
  ];
  return (
    <div className="flex flex-1 flex-col gap-4 overflow-scroll pb-2">
      <Table
        columns={columns}
        data={assets}
        selectColumns
        showFiltering
        enablePagination
        currentPage={currentPage}
        setCurrentPage={setCurrentPage}
        totalPages={totalPages}
        resultsPerPage={resultsPerPage}
        sortOptions={sortOptions}
        currentSortValue={currentSortValue}
        handleSortChange={handleSortChange}
      />
    </div>
  );
};

export { AssetsListView };
