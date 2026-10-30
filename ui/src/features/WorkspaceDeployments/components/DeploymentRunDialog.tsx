import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  DialogFooter,
} from "@flow/components";
import { missingRequiredVariables } from "@flow/features/WorkspaceTriggers/components/TriggerWorkflowVariables/triggerVariables";
import { useT } from "@flow/lib/i18n";
import type { Deployment } from "@flow/types";

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
  // A manual run takes the deployment's defaults as they are, so a required
  // variable without one would have the run refused.
  const missing = missingRequiredVariables(deployment.parameters, undefined);

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
            {missing.length > 0 && (
              <p className="mt-2 text-xs text-destructive">
                {t(
                  "This deployment can't run: these required variables have no default: {{names}}. Give them a default in the project and deploy again, or run it from a trigger that sets them.",
                  { names: missing.join(", ") },
                )}
              </p>
            )}
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button variant="outline" size="sm" onClick={onDialogClose}>
            {t("Cancel")}
          </Button>
          <Button
            variant="default"
            size="sm"
            disabled={missing.length > 0}
            onClick={handleDeploymentRun}>
            {t("Run")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { DeploymentRunDialog };
