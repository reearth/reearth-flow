import { ClipboardTextIcon, TrashIcon } from "@phosphor-icons/react";
import { DotsHorizontalIcon } from "@radix-ui/react-icons";
import { ColumnDef } from "@tanstack/react-table";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@flow/components";
import { DataTable as Table } from "@flow/components/DataTable";
import { useToast } from "@flow/features/NotificationSystem/useToast";
import { DEPLOYMENT_FETCH_RATE } from "@flow/lib/gql/deployment/useQueries";
import { useT } from "@flow/lib/i18n";
import type { Asset } from "@flow/types";
import { copyToClipboard } from "@flow/utils/copyToClipboard";

type Props = {
  assets?: Asset[];
  isFetching: boolean;
  currentPage: number;
  totalPages: number;
  sortOptions: { value: string; label: string }[];
  currentSortValue: string;
  handleSortChange: (value: string) => void;
  setCurrentPage?: (page: number) => void;
  setAssetToBeDeleted: (asset: string | undefined) => void;
  searchTerm?: string;
  setSearchTerm: (term: string) => void;
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
  searchTerm,
  setSearchTerm,
}) => {
  const t = useT();
  const { toast } = useToast();

  const handleCopyURLToClipBoard = (url: string) => {
    if (!url) return;
    copyToClipboard(url);
    toast({
      title: t("Copied to clipboard"),
      description: t("{{asset}} asset's URL copied to clipboard", {
        resource: name,
      }),
    });
  };
  const resultsPerPage = DEPLOYMENT_FETCH_RATE;
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
      accessorKey: "url",
      header: t("Path"),
    },
    {
      accessorKey: "quickActions",
      header: t("Actions"),
      cell: (row) => (
        <DropdownMenu modal={false}>
          <DropdownMenuTrigger
            className="flex h-full w-[40px] items-center justify-center rounded-md hover:bg-accent"
            onClick={(e) => e.stopPropagation()}>
            <DotsHorizontalIcon className="size-[24px]" />
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" onClick={(e) => e.stopPropagation()}>
            <DropdownMenuItem
              className="justify-between gap-2"
              disabled={!row.row.original.url}
              onClick={(e) => {
                e.stopPropagation();
                handleCopyURLToClipBoard(row.row.original.url);
              }}>
              {t("Copy Asset URL")}
              <ClipboardTextIcon weight="light" />
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              className="justify-between gap-4 text-destructive"
              onClick={(e) => {
                e.stopPropagation();
                setAssetToBeDeleted(row.row.original.id);
              }}>
              {t("Delete Asset")}
              <TrashIcon weight="light" />
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      ),
    },
  ];
  return (
    <div className="overflow-scroll">
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
        searchTerm={searchTerm}
        setSearchTerm={setSearchTerm}
      />
    </div>
  );
};

export { AssetsListView };
