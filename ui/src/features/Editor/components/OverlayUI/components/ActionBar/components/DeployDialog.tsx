// import { CaretRight } from "@phosphor-icons/react";
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
    deploymentId?: string,
    description?: string,
  ) => Promise<void>;
  setShowDialog: (show: boolean) => void;
};

const DeployDialog: React.FC<Props> = ({
  allowedToDeploy,
  onWorkflowDeployment,
  setShowDialog,
}) => {
  const t = useT();
  const [currentProject] = useCurrentProject();

  const deployment = useMemo(
    () => currentProject?.deployment,
    [currentProject?.deployment],
  );

  const [description, setDescription] = useState<string>(
    deployment?.description ?? "",
  );

  const handleWorkflowDeployment = useCallback(async () => {
    await onWorkflowDeployment(deployment?.id, description);
    if (allowedToDeploy) {
      setShowDialog(false);
    }
  }, [
    description,
    deployment?.id,
    allowedToDeploy,
    onWorkflowDeployment,
    setShowDialog,
  ]);

  return (
    <Dialog open={true} onOpenChange={() => setShowDialog(false)}>
      <DialogContent size="sm">
        <DialogTitle>{t("Deploy project")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex flex-col">
            <Label>{t("Project to deploy: ")}</Label>
            <p className="truncate dark:font-thin">
              {currentProject?.name ?? t("N/A")}
            </p>
          </DialogContentSection>
          {/* <DialogContentSection>
            <Label>{t("Deploy version: ")}</Label>
            <div className="flex items-center">
              <p className="dark:font-thin">{deployment?.version || 1.0}</p>
              <CaretRight />
              <p className="font-semibold">2.0</p>
            </div>
          </DialogContentSection> */}
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
