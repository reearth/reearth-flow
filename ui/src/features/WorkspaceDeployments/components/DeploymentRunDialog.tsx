import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  DialogFooter,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Deployment } from "@flow/types";

type Props = {
  deployment: Deployment;
  onDeploymentRun: (deployment: Deployment) => Promise<void>;
  onDialogClose: () => void;
};

const DeploymentRunDialog: React.FC<Props> = ({
  deployment,
  onDeploymentRun,
  onDialogClose,
}) => {
  const t = useT();

  const handleDeploymentRun = async () => {
    await onDeploymentRun(deployment);
    onDialogClose();
  };

  return (
    <Dialog open={true} onOpenChange={onDialogClose}>
      <DialogContent size="xs">
        <DialogTitle>{t("Run Deployment")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection>
            <p className="dark:font-light">
              {t("Are you sure you want to run this deployment?")}
            </p>
            {deployment.description && (
              <p className="mt-2 text-sm text-muted-foreground">
                {deployment.description}
              </p>
            )}
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button variant="outline" size="sm" onClick={onDialogClose}>
            {t("Cancel")}
          </Button>
          <Button variant="default" size="sm" onClick={handleDeploymentRun}>
            {t("Run")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { DeploymentRunDialog };
