import { CaretLeft, PencilLine, Trash } from "@phosphor-icons/react";
import { useRouter } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";

import { Button } from "@flow/components";
import { DetailsBox, DetailsBoxContent } from "@flow/features/common";
// import { LogConsole } from "@flow/features/Editor/components/BottomPanel/components";
import { useT } from "@flow/lib/i18n";
import { Trigger } from "@flow/types/trigger";

import { TriggerEditDialog } from "./TriggerEditDialog";

type Props = {
  selectedTrigger?: Trigger;
  setTriggerToBeDeleted: (trigger?: Trigger) => void;
};

const TriggerDetails: React.FC<Props> = ({
  selectedTrigger,
  setTriggerToBeDeleted,
}) => {
  const t = useT();
  const { history } = useRouter();
  const [openDeploymentEditDialog, setOpenDeploymentEditDialog] =
    useState(false);

  const handleBack = useCallback(() => history.go(-1), [history]); // Go back to previous page

  const details: DetailsBoxContent[] | undefined = useMemo(
    () =>
      selectedTrigger
        ? [
            {
              id: "id",
              name: t("ID"),
              value: selectedTrigger.id,
            },
            {
              id: "deployment",
              name: t("Deployment Id"),
              value: selectedTrigger.deployment,
            },
            {
              id: "authToken",
              name: t("Auth Token"),
              value: selectedTrigger.authToken,
            },
            {
              id: "timeInterval",
              name: t("Time Interval"),
              value: selectedTrigger.timeInterval,
            },
            {
              id: "eventSource",
              name: t("Event Source"),
              value: selectedTrigger.eventSource,
            },
            {
              id: "lastTriggered",
              name: t("Last Triggered"),
              value: selectedTrigger.lastTriggered,
            },
            {
              id: "createdAt",
              name: t("Created At"),
              value: selectedTrigger.createdAt,
            },
            {
              id: "updatedAt",
              name: t("Updated At"),
              value: selectedTrigger.updatedAt,
            },
          ]
        : undefined,
    [t, selectedTrigger],
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
              variant="outline"
              size="sm"
              disabled={!selectedTrigger}
              onClick={() => setOpenDeploymentEditDialog(true)}>
              <PencilLine />
              {t("Edit Trigger")}
            </Button>
            <Button
              variant="destructive"
              size="sm"
              onClick={() => setTriggerToBeDeleted(selectedTrigger)}>
              <Trash />
              {t("Delete")}
            </Button>
          </div>
        </div>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
          <DetailsBox title={t("Trigger Details")} content={details} />
        </div>
      </div>
      {openDeploymentEditDialog && selectedTrigger && (
        <TriggerEditDialog
          selectedTrigger={selectedTrigger}
          onDialogClose={() => setOpenDeploymentEditDialog(false)}
        />
      )}
    </>
  );
};

export { TriggerDetails };
