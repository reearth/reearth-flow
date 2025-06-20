import { useNavigate } from "@tanstack/react-router";
import { useCallback, useEffect, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  Label,
  DialogFooter,
} from "@flow/components";
import { DeploymentsDialog } from "@flow/features/WorkspaceDeployments/components/DeploymentsDialog";
import { useDeployment } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import type { Deployment } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";

type Props = {
  setShowDialog: (show: boolean) => void;
};

const JobRunDialog: React.FC<Props> = ({ setShowDialog }) => {
  const t = useT();
  const navigate = useNavigate();
  const [selectedDeployment, selectDeployment] = useState<
    Deployment | undefined
  >(undefined);
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [currentOrder, setCurrentOrder] = useState<OrderDirection>(
    OrderDirection.Desc,
  );
  const [openSelectDeploymentsDialog, setOpenSelectDeploymentsDialog] =
    useState<boolean>(false);
  const [currentWorkspace] = useCurrentWorkspace();

  const { useGetDeployments, executeDeployment } = useDeployment();
  const { page, isFetching, refetch } = useGetDeployments(currentWorkspace?.id);

  useEffect(() => {
    (async () => {
      await refetch();
    })();
  }, [currentPage, currentOrder, refetch]);

  const deployments = page?.deployments;
  const totalPages = page?.totalPages as number;

  const handleJob = useCallback(async () => {
    if (!selectedDeployment || !currentWorkspace) return;
    const jobData = await executeDeployment({
      deploymentId: selectedDeployment.id,
    });
    if (jobData) {
      navigate({
        to: `/workspaces/${currentWorkspace.id}/jobs/${jobData.job?.id}`,
      });
    }
  }, [currentWorkspace, selectedDeployment, navigate, executeDeployment]);

  return (
    <Dialog open={true} onOpenChange={() => setShowDialog(false)}>
      <DialogContent size="sm">
        <DialogTitle>{t("Run a deployment")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex flex-col">
            <Label>{t("Deployment: ")}</Label>
            <div
              className="flex h-8 w-full rounded-md border bg-transparent px-3 py-1 text-sm"
              onClick={() => setOpenSelectDeploymentsDialog(true)}>
              <span className="cursor-default pr-2 whitespace-nowrap text-muted-foreground">
                {t("Select Deployment: ")}
              </span>
              {selectedDeployment ? (
                <span className="cursor-default truncate">
                  {selectedDeployment.description} @{selectedDeployment.version}
                </span>
              ) : (
                <span className="cursor-default">
                  {t("No Deployment Selected")}
                </span>
              )}
            </div>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            className="self-end"
            variant="outline"
            disabled={!selectedDeployment}
            onClick={handleJob}>
            {t("Run Deployment")}
          </Button>
        </DialogFooter>
      </DialogContent>
      {openSelectDeploymentsDialog && (
        <DeploymentsDialog
          deployments={deployments}
          currentPage={currentPage}
          totalPages={totalPages}
          currentOrder={currentOrder}
          isFetching={isFetching}
          setShowDialog={() => setOpenSelectDeploymentsDialog(false)}
          handleSelectDeployment={selectDeployment}
          setCurrentPage={setCurrentPage}
          setCurrentOrder={setCurrentOrder}
        />
      )}
    </Dialog>
  );
};

export { JobRunDialog };
