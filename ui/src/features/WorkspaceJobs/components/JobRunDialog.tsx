import { Plus } from "@phosphor-icons/react";
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
    refetch();
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
          <DialogContentSection className="flex-1">
            <Label htmlFor="deployments-selector">
              {t("Select a deployment")}
            </Label>
            <Button
              variant={selectedDeployment ? "default" : "outline"}
              size="sm"
              onClick={() => setOpenSelectDeploymentsDialog(true)}>
              {!selectedDeployment && <Plus />}
              {selectedDeployment
                ? `${
                    (selectedDeployment?.description?.length ?? 0) > 20
                      ? `${selectedDeployment?.description?.substring(0, 20)}...`
                      : selectedDeployment.description
                  } @${selectedDeployment.version}`
                : t("Select a deployment")}
            </Button>
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
          setShowDialog={() => setOpenSelectDeploymentsDialog(false)}
          deployments={deployments}
          handleSelectDeployment={selectDeployment}
          currentPage={currentPage}
          setCurrentPage={setCurrentPage}
          totalPages={totalPages}
          currentOrder={currentOrder}
          setCurrentOrder={setCurrentOrder}
          isFetching={isFetching}
        />
      )}
    </Dialog>
  );
};

export { JobRunDialog };
