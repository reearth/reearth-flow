import {
  ArrowRightIcon,
  ArrowSquareOutIcon,
  CheckCircleIcon,
  RocketIcon,
  WarningCircleIcon,
} from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";

import { Button, Input, Label } from "@flow/components";
import type { DeploymentChangeStatus } from "@flow/hooks/useDeploymentChanges";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";
import type { Deployment } from "@flow/types";

type Props = {
  allowedToDeploy: boolean;
  deploymentChangeStatus?: DeploymentChangeStatus;
  onWorkflowDeployment: (
    description: string,
    deploymentId?: string,
  ) => Promise<Deployment | undefined>;
  onDialogClose: () => void;
};

const DeployPopover: React.FC<Props> = ({
  allowedToDeploy,
  deploymentChangeStatus,
  onWorkflowDeployment,
  onDialogClose,
}) => {
  const t = useT();
  const navigate = useNavigate();
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
  const [isDeploying, setIsDeploying] = useState(false);
  // Set once a (re)deployment succeeds, so the popover can stay open and
  // confirm it.
  const [result, setResult] = useState<
    { deployment: Deployment; isUpdate: boolean } | undefined
  >(undefined);

  const handleWorkflowDeployment = useCallback(async () => {
    setIsDeploying(true);
    try {
      const deployed = await onWorkflowDeployment(description, deployment?.id);
      if (deployed) {
        setResult({ deployment: deployed, isUpdate: !!deployment });
      }
    } finally {
      setIsDeploying(false);
    }
  }, [description, deployment, onWorkflowDeployment]);

  const deploymentToView = result?.deployment ?? deployment;

  const handleViewDetails = useCallback(() => {
    if (!deploymentToView) return;
    onDialogClose();
    navigate({
      to: `/workspaces/${deploymentToView.workspaceId}/deployments/${deploymentToView.id}`,
    });
  }, [deploymentToView, navigate, onDialogClose]);

  return (
    <div className="flex flex-col gap-4 p-4">
      <h4 className="text-md flex items-center gap-2 leading-none tracking-tight dark:font-thin">
        <RocketIcon weight="thin" size={18} />
        {t("Deploy Project")}
      </h4>
      {result ? (
        <>
          <div className="flex items-start gap-2">
            <CheckCircleIcon
              className="mt-px shrink-0 text-success"
              weight="fill"
              size={18}
            />
            <p className="text-sm dark:font-light">
              {result.isUpdate
                ? t("Deployment has been successfully updated.")
                : t("Deployment has been successfully created.")}
            </p>
          </div>
          <div className="flex items-center justify-between">
            <Label>{t("Deployment Version: ")}</Label>
            <p className="text-sm font-semibold">{result.deployment.version}</p>
          </div>
          <div className="flex justify-end">
            <Button
              className="flex gap-2"
              variant="outline"
              onClick={handleViewDetails}>
              <ArrowSquareOutIcon weight="thin" />
              {t("View Details")}
            </Button>
          </div>
        </>
      ) : (
        <>
          <div className="flex items-center justify-between">
            <Label>{t("Deployment Version: ")}</Label>
            <div className="flex items-center gap-1.5 text-sm">
              {currentVersion && (
                <>
                  <span className="text-muted-foreground">
                    v{currentVersion}
                  </span>
                  <ArrowRightIcon className="text-muted-foreground" />
                </>
              )}
              <span className="font-semibold">
                v{currentVersion ? currentVersion + 1 : 1}
              </span>
            </div>
          </div>
          {deployment && deploymentChangeStatus && (
            <div className="-mt-2 flex items-center gap-1.5 text-xs">
              {deploymentChangeStatus === "changed" ? (
                <>
                  <WarningCircleIcon className="shrink-0 text-warning" />
                  <span>
                    {t("Changes since {{version}}", {
                      version: deployment.version,
                    })}
                  </span>
                </>
              ) : (
                <>
                  <CheckCircleIcon className="shrink-0 text-muted-foreground" />
                  <span className="text-muted-foreground">
                    {t("No changes since {{version}}", {
                      version: deployment.version,
                    })}
                  </span>
                </>
              )}
            </div>
          )}
          <div className="flex flex-col gap-2">
            <Label>{t("Description")}</Label>
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder={t(
                "Give your deployment a meaningful description...",
              )}
            />
          </div>
          <div className="flex items-center justify-between gap-2">
            {deployment ? (
              <Button
                className="-ml-3 flex gap-2 text-muted-foreground"
                variant="ghost"
                onClick={handleViewDetails}>
                <ArrowSquareOutIcon weight="thin" />
                {t("View Details")}
              </Button>
            ) : (
              <span />
            )}
            <Button
              className="shrink-0"
              variant="outline"
              disabled={!allowedToDeploy || isDeploying || !description.trim()}
              onClick={handleWorkflowDeployment}>
              {deployment ? t("Update") : t("Deploy")}
            </Button>
          </div>
        </>
      )}
    </div>
  );
};

export default DeployPopover;
