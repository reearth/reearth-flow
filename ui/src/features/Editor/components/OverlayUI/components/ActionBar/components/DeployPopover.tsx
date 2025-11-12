import { CaretRightIcon } from "@phosphor-icons/react";
import { useCallback, useMemo, useState } from "react";

import { Button, Input, Label } from "@flow/components";
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

const DeployPopover: React.FC<Props> = ({
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
    <div className="flex flex-col gap-2 p-4">
      <div className="flex justify-between gap-2">
        <h4 className="text-md self-center rounded-t-lg leading-none tracking-tight dark:font-thin">
          {t("Deploy Project")}
        </h4>
      </div>
      <div className="flex flex-col gap-2">
        <div className="flex flex-row items-center">
          <Label>{t("Deployment Version: ")}</Label>
          <div className="flex items-center gap-2">
            <p className="pl-1 dark:font-thin">{currentVersion}</p>
            <CaretRightIcon />
            <p className="font-semibold">
              {currentVersion ? currentVersion + 1 : 1}
            </p>
          </div>
        </div>
        <div className="flex flex-col gap-2">
          <Label>{t("Description")}</Label>
          <Input
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder={t("Give your deployment a meaningful description...")}
          />
        </div>
        <div className="flex flex-col gap-4">
          <p className="text-sm dark:font-light">
            {t("Are you sure you want to proceed?")}
          </p>
          <div className="flex items-center justify-end">
            <Button
              variant="outline"
              disabled={!description.trim()}
              onClick={handleWorkflowDeployment}>
              {deployment ? t("Update") : t("Deploy")}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default DeployPopover;
