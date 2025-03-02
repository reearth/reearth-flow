import { CaretLeft, PencilLine, Play, Trash } from "@phosphor-icons/react";
import { useRouter } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";

import { Button } from "@flow/components";
import { DetailsBox, DetailsBoxContent } from "@flow/features/common";
// import { LogConsole } from "@flow/features/Editor/components/BottomPanel/components";
import { useT } from "@flow/lib/i18n";
import type { Deployment } from "@flow/types";
import { formatTimestamp } from "@flow/utils";

import { DeploymentEditDialog } from "./DeploymentEditDialog";

type Props = {
  selectedDeployment?: Deployment;
  setDeploymentToBeDeleted: (deployment?: Deployment) => void;
  onDeploymentRun: (deployment?: Deployment) => Promise<void>;
};

const DeploymentDetails: React.FC<Props> = ({
  selectedDeployment,
  setDeploymentToBeDeleted,
  onDeploymentRun,
}) => {
  const t = useT();
  const { history } = useRouter();
  const [openDeploymentEditDialog, setOpenDeploymentEditDialog] =
    useState(false);

  const handleBack = useCallback(() => history.go(-1), [history]); // Go back to previous page

  const details: DetailsBoxContent[] | undefined = useMemo(
    () =>
      selectedDeployment
        ? [
            {
              id: "id",
              name: t("ID"),
              value:
                selectedDeployment.id || t("Unknown or deleted deployment"),
            },
            {
              id: "description",
              name: t("Description"),
              value: selectedDeployment.description || t("N/A"),
            },
            {
              id: "project",
              name: t("Project Name"),
              value:
                selectedDeployment.projectName ||
                t("Unknown or deleted project"),
            },
            {
              id: "version",
              name: t("Version"),
              value: selectedDeployment.version || "",
            },
            {
              id: "createdAt",
              name: t("Created At"),
              value:
                formatTimestamp(selectedDeployment.createdAt) ||
                t("Never") ||
                "",
            },
            {
              id: "updatedAt",
              name: t("Updated At"),
              value:
                formatTimestamp(selectedDeployment.updatedAt) ||
                t("Never") ||
                "",
            },
            {
              id: "workflowDownload",
              name: t("Workflow Download"),
              value: selectedDeployment.workflowUrl,
              type: "download",
            },
          ]
        : undefined,
    [t, selectedDeployment],
  );

  return (
    <>
      <div className="flex flex-1 flex-col gap-4 px-6 pb-2 pt-6">
        <div className="flex justify-between">
          <Button size="icon" variant="ghost" onClick={handleBack}>
            <CaretLeft />
          </Button>
          <div className="flex gap-2">
            <Button
              variant="default"
              size="sm"
              onClick={() => onDeploymentRun(selectedDeployment)}>
              <Play />
              {t("Run")}
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={!selectedDeployment}
              onClick={() => setOpenDeploymentEditDialog(true)}>
              <PencilLine />
              {t("Edit Deployment")}
            </Button>
            <Button
              variant="destructive"
              size="sm"
              onClick={() => setDeploymentToBeDeleted(selectedDeployment)}>
              <Trash />
              {t("Delete")}
            </Button>
          </div>
        </div>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
          <DetailsBox title={t("Deployment Details")} content={details} />
        </div>
      </div>
      {openDeploymentEditDialog && selectedDeployment && (
        <DeploymentEditDialog
          selectedDeployment={selectedDeployment}
          onDialogClose={() => setOpenDeploymentEditDialog(false)}
        />
      )}
    </>
  );
};

export { DeploymentDetails };
