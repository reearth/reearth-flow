import { ColumnDef } from "@tanstack/react-table";

import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  DataTable as Table,
  FlowLogo,
  LoadingSkeleton,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { DEPLOYMENT_FETCH_RATE } from "@flow/lib/gql/deployment/useQueries";
import { useT } from "@flow/lib/i18n";
import { Deployment } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";

type Props = {
  deployments: Deployment[] | undefined;
  currentPage?: number;
  currentOrder?: OrderDirection;
  totalPages?: number;
  isFetching?: boolean;
  setShowDialog: (show: boolean) => void;
  handleSelectDeployment: (deployment: Deployment) => void;
  setCurrentPage?: (page: number) => void;
  setCurrentOrder?: (order: OrderDirection) => void;
};

const DeploymentsDialog: React.FC<Props> = ({
  deployments,
  currentPage = 1,
  currentOrder = OrderDirection.Desc,
  totalPages,
  isFetching,
  setShowDialog,
  handleSelectDeployment,
  setCurrentPage,
  setCurrentOrder,
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
            {isFetching ? (
              <LoadingSkeleton className="h-[373px]" />
            ) : deployments && deployments.length > 0 ? (
              <Table
                columns={columns}
                data={deployments}
                selectColumns
                enablePagination
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
                text={t("No Deployments")}
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
