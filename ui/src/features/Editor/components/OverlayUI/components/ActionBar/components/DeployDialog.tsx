import { CaretRightIcon } from "@phosphor-icons/react";
import { useCallback, useMemo, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  Label,
  DialogFooter,
  Input,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";

type Props = {
  allowedToDeploy: boolean;
  onWorkflowDeployment: (
    description: string,
    deploymentId?: string,
  ) => Promise<void>;
  onDialogClose: () => void;
};

const DeployDialog: React.FC<Props> = ({
  allowedToDeploy,
  onWorkflowDeployment,
  onDialogClose,
}) => {
  const t = useT();
  const [currentProject] = useCurrentProject();

  const deployment = useMemo(
    () => currentProject?.deployment,
    [currentProject?.deployment],
  );

  const currentVersion = useMemo(() => {
    if (!deployment) return undefined;
    const versionNumber = parseInt(deployment.version.slice(1));
    if (Number.isNaN(versionNumber)) return undefined;
    return versionNumber;
  }, [deployment]);

  const [description, setDescription] = useState<string>(
    deployment?.description ?? "",
  );

  const handleWorkflowDeployment = useCallback(async () => {
    await onWorkflowDeployment(description, deployment?.id);
    if (allowedToDeploy) {
      onDialogClose();
    }
  }, [
    description,
    deployment?.id,
    allowedToDeploy,
    onWorkflowDeployment,
    onDialogClose,
  ]);

  return (
    <Dialog open={true} onOpenChange={onDialogClose}>
      <DialogContent size="sm">
        <DialogTitle>{t("Deploy Project")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex flex-row items-center">
            <Label>{t("Project to Deploy: ")}</Label>
            <p className="truncate dark:font-thin">
              {currentProject?.name ?? t("N/A")}
            </p>
          </DialogContentSection>
          <DialogContentSection className="flex flex-row items-center">
            <Label>{t("Deployment Version: ")}</Label>
            <div className="flex items-center gap-2">
              <p className="dark:font-thin">{currentVersion}</p>
              <CaretRightIcon />
              <p className="font-semibold">
                {currentVersion ? currentVersion + 1 : 1}
              </p>
            </div>
          </DialogContentSection>
          <div className="border-t border-primary" />
          <DialogContentSection className="flex flex-col">
            <Label>{t("Description")}</Label>
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder={t(
                "Give your deployment a meaningful description...",
              )}
            />
          </DialogContentSection>
          <DialogContentSection>
            <p className="dark:font-light">
              {t("Are you sure you want to proceed?")}
            </p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            disabled={!description.trim()}
            onClick={handleWorkflowDeployment}>
            {deployment ? t("Update") : t("Deploy")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default DeployDialog;
