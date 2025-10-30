import { ColumnDef } from "@tanstack/react-table";

import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  DataTable as Table,
} from "@flow/components";
import { DEPLOYMENT_FETCH_RATE } from "@flow/lib/gql/deployment/useQueries";
import { useT } from "@flow/lib/i18n";
import { Deployment } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";

type Props = {
  deployments: Deployment[] | undefined;
  currentPage?: number;
  totalPages?: number;
  isFetching?: boolean;
  currentSortValue?: string;
  sortOptions?: { value: string; label: string }[];
  onSelectDeployment: (deployment: Deployment) => void;
  onSortChange?: (value: string) => void;
  setCurrentPage?: (page: number) => void;
  setCurrentOrderDir?: (order: OrderDirection) => void;
  setSearchTerm?: (term: string) => void;
  setShowDialog: (show: boolean) => void;
};

const DeploymentsDialog: React.FC<Props> = ({
  deployments,
  currentPage = 1,
  sortOptions,
  currentSortValue,
  totalPages,
  isFetching,
  onSelectDeployment,
  onSortChange,
  setCurrentPage,
  setCurrentOrderDir,
  setSearchTerm,
  setShowDialog,
}) => {
  const t = useT();
  const resultsPerPage = DEPLOYMENT_FETCH_RATE;
  const columns: ColumnDef<Deployment>[] = [
    {
      accessorKey: "description",
      header: t("Description"),
    },
    {
      accessorKey: "projectName",
      header: t("Project Name"),
    },
    {
      accessorKey: "version",
      header: t("Version"),
    },
    {
      accessorKey: "updatedAt",
      header: t("Updated At"),
    },
  ];

  return (
    <Dialog open={true} onOpenChange={() => setShowDialog(false)}>
      <DialogContent size="xl" className="min-h-96">
        <DialogTitle> {t("Select a deployment")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex-1">
            <Table
              columns={columns}
              data={deployments}
              selectColumns
              enablePagination
              currentPage={currentPage}
              totalPages={totalPages}
              resultsPerPage={resultsPerPage}
              currentSortValue={currentSortValue}
              sortOptions={sortOptions}
              showFiltering
              isFetching={isFetching}
              noResultsMessage={t("No Deployments")}
              onRowClick={(deployment) => {
                onSelectDeployment(deployment);
                setShowDialog(false);
              }}
              onSortChange={onSortChange}
              setCurrentPage={setCurrentPage}
              setCurrentOrderDir={setCurrentOrderDir}
              setSearchTerm={setSearchTerm}
            />
          </DialogContentSection>
        </DialogContentWrapper>
      </DialogContent>
    </Dialog>
  );
};

export { DeploymentsDialog };
