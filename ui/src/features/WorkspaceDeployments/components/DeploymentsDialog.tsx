import { ColumnDef } from "@tanstack/react-table";

import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  DataTable as Table,
  FlowLogo,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { DEPLOYMENT_FETCH_RATE } from "@flow/lib/gql/deployment/useQueries";
import { useT } from "@flow/lib/i18n";
import { Deployment } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";

type Props = {
  setShowDialog: (show: boolean) => void;
  deployments: Deployment[] | undefined;
  handleSelectDeployment: (deployment: Deployment) => void;
  currentPage?: number;
  setCurrentPage?: (page: number) => void;
  currentOrder?: OrderDirection;
  setCurrentOrder?: (order: OrderDirection) => void;
  totalPages?: number;
  isFetching?: boolean;
};

const DeploymentsDialog: React.FC<Props> = ({
  setShowDialog,
  deployments,
  handleSelectDeployment,
  currentPage = 1,
  setCurrentPage,
  currentOrder = OrderDirection.Desc,
  setCurrentOrder,
  totalPages,
  isFetching,
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
      <DialogContent size="md">
        <DialogTitle> {t("Select a deployment")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex-1">
            {isFetching ? (
              <span>{t("Loading")}</span>
            ) : deployments && deployments.length > 0 ? (
              <Table
                columns={columns}
                data={deployments}
                selectColumns
                showFiltering
                enablePagination
                rowHeight={14}
                onRowClick={(deployment) => {
                  handleSelectDeployment(deployment);
                  setShowDialog(false);
                }}
                currentPage={currentPage}
                setCurrentPage={setCurrentPage}
                totalPages={totalPages}
                resultsPerPage={resultsPerPage}
                currentOrder={currentOrder}
                setCurrentOrder={setCurrentOrder}
              />
            ) : (
              <BasicBoiler
                text={t("No Jobs")}
                icon={<FlowLogo className="size-16 text-accent" />}
              />
            )}
          </DialogContentSection>
        </DialogContentWrapper>
      </DialogContent>
    </Dialog>
  );
};

export { DeploymentsDialog };
