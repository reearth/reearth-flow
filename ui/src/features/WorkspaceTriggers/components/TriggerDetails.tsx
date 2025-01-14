import { CaretLeft } from "@phosphor-icons/react";
import { useRouter } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";

import { Button } from "@flow/components";
import { DetailsBox, DetailsBoxContent } from "@flow/features/common";
import { useT } from "@flow/lib/i18n";
import type { Deployment } from "@flow/types";

type Props = {
  selectedDeployment?: Deployment;
  setDeploymentToBeDeleted: (deployment: string | undefined) => void;
  onDeploymentUpdate: (description?: string) => Promise<void>;
};

const TriggerDetails: React.FC<Props> = ({
  selectedDeployment,
  setDeploymentToBeDeleted,
  onDeploymentUpdate,
}) => {
  const t = useT();
  const { history } = useRouter();

  const [updatedDescription, setUpdatedDescription] = useState(
    selectedDeployment?.description || "",
  );

  const handleBack = useCallback(() => history.go(-1), [history]); // Go back to previous page

  const handleUpdate = useCallback(
    () => onDeploymentUpdate(updatedDescription),
    [onDeploymentUpdate, updatedDescription],
  );

  const handleDelete = useCallback(() => {
    if (!selectedDeployment) return;
    setDeploymentToBeDeleted(selectedDeployment.id);
  }, [selectedDeployment, setDeploymentToBeDeleted]);

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
              id: "deployment",
              name: t("Deployment"),
              value:
                selectedDeployment.projectName ||
                t("Unknown or deleted project"),
            },
            {
              id: "eventSource",
              name: t("Event Source"),
              value: selectedDeployment.updatedAt,
              type: "textbox",
            },
            {
              id: "timeInterval",
              name: t("Time Interval"),
              value: selectedDeployment.updatedAt,
            },
            {
              id: "lastTriggered",
              name: t("Last Triggered"),
              value: selectedDeployment.updatedAt,
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
            {t("Delete Trigger")}
          </Button>
        </div>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
          <DetailsBox
            title={t("Trigger Details")}
            content={details}
            onContentChange={handleDescriptionChange}
          />
          <Button
            variant="default"
            className="self-end"
            disabled={updatedDescription === selectedDeployment.description}
            onClick={handleUpdate}>
            {t("Update Trigger")}
          </Button>
        </div>
      </div>
    )
  );
};

export { TriggerDetails };
