import { CaretLeft } from "@phosphor-icons/react";
import { useRouter } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";

import { Button } from "@flow/components";
import { DetailsBox, DetailsBoxContent } from "@flow/features/common";
// import { LogConsole } from "@flow/features/Editor/components/BottomPanel/components";
import { useT } from "@flow/lib/i18n";
import type { Deployment } from "@flow/types";

type Props = {
  selectedDeployment?: Deployment;
  onDeploymentUpdate: (description?: string) => Promise<void>;
  onDeploymentDelete: () => void;
};

const DeploymentDetails: React.FC<Props> = ({
  selectedDeployment,
  onDeploymentUpdate,
  onDeploymentDelete,
}) => {
  const t = useT();
  const { history } = useRouter();

  const [updatedDescription, setUpdatedDescription] = useState(
    selectedDeployment?.description || "",
  );

  const handleBack = useCallback(() => history.go(-1), [history]);

  const handleUpdate = useCallback(() => {
    onDeploymentUpdate(updatedDescription);
  }, [onDeploymentUpdate, updatedDescription]);

  const handleDelete = useCallback(() => {
    if (!selectedDeployment) return;
    onDeploymentDelete();
    handleBack();
  }, [selectedDeployment, handleBack, onDeploymentDelete]);

  const handleDescriptionChange = useCallback((content: DetailsBoxContent) => {
    setUpdatedDescription(content.value);
  }, []);

  const details: DetailsBoxContent[] | undefined = useMemo(
    () =>
      selectedDeployment
        ? [
            {
              id: "id",
              name: t("ID"),
              value: selectedDeployment.id,
            },
            {
              id: "description",
              name: t("Description"),
              value: updatedDescription,
              type: "textbox",
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
              value: selectedDeployment.version,
            },
            {
              id: "createdAt",
              name: t("Created At"),
              value: selectedDeployment.createdAt,
            },
            {
              id: "updatedAt",
              name: t("Updated At"),
              value: selectedDeployment.updatedAt,
            },
            {
              id: "workflowUrl",
              name: t("Workflow Url"),
              value: selectedDeployment.workflowUrl,
              type: "download",
            },
          ]
        : undefined,
    [t, selectedDeployment, updatedDescription],
  );

  return (
    selectedDeployment && (
      <div className="flex flex-1 flex-col gap-4 px-6 pb-2 pt-6">
        <div className="flex justify-between">
          <Button size="icon" variant="ghost" onClick={handleBack}>
            <CaretLeft />
          </Button>
          <Button variant="destructive" onClick={handleDelete}>
            {t("Delete Deployment")}
          </Button>
        </div>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
          <DetailsBox
            title={t("Deployment Details")}
            content={details}
            onContentChange={handleDescriptionChange}
          />
          <Button
            variant="default"
            className="self-end"
            disabled={updatedDescription === selectedDeployment.description}
            onClick={handleUpdate}>
            {t("Update Deployment")}
          </Button>
        </div>
      </div>
    )
  );
};

export { DeploymentDetails };
