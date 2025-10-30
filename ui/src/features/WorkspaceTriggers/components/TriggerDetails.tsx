import {
  CaretLeftIcon,
  CopyIcon,
  PencilLineIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { useRouter } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";

import { Button, IconButton } from "@flow/components";
import { config } from "@flow/config";
import { DetailsBox, DetailsBoxContent } from "@flow/features/common";
import { useT } from "@flow/lib/i18n";
import { Trigger } from "@flow/types";
import { formatTimestamp } from "@flow/utils";

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
  const [openTriggerEditDialog, setOpenTriggerEditDialog] = useState(false);

  const handleBack = useCallback(() => history.go(-1), [history]); // Go back to previous page
  const apiUrl = config().api || window.location.origin;
  const details: DetailsBoxContent[] | undefined = useMemo(
    () =>
      selectedTrigger
        ? [
            {
              id: "id",
              name: t("ID"),
              value: selectedTrigger.id || t("Unknown or deleted trigger"),
            },
            {
              id: "triggerId",
              name: t("Trigger Description"),
              value: selectedTrigger.description || t("N/A"),
            },
            {
              id: "deploymentId",
              name: t("Deployment Id"),
              value: selectedTrigger.deploymentId || t("N/A"),
            },
            {
              id: "projectName",
              name: t("Project Name"),
              value:
                selectedTrigger.deployment.projectName ||
                t("Unknown or deleted project"),
            },
            {
              id: "deploymentDescription",
              name: t("Deployment Description"),
              value: selectedTrigger.deployment.description || t("N/A"),
            },
            {
              id: "eventSource",
              name: t("Event Source"),
              value: selectedTrigger.eventSource,
            },
            ...(selectedTrigger.eventSource === "API_DRIVEN"
              ? [
                  {
                    id: "authToken",
                    name: t("Auth Token"),
                    value: selectedTrigger.authToken || t("N/A"),
                  },
                ]
              : []),
            ...(selectedTrigger.eventSource === "TIME_DRIVEN"
              ? [
                  {
                    id: "timeInterval",
                    name: t("Time Interval"),
                    value: selectedTrigger.timeInterval || t("N/A"),
                  },
                ]
              : []),
            {
              id: "lastTriggered",
              name: t("Last Triggered"),
              value: selectedTrigger.lastTriggered || t("Never"),
            },
            {
              id: "createdAt",
              name: t("Created At"),
              value:
                formatTimestamp(selectedTrigger.createdAt) || t("Never") || "",
            },
            {
              id: "updatedAt",
              name: t("Updated At"),
              value:
                formatTimestamp(selectedTrigger.updatedAt) || t("Never") || "",
            },
            {
              id: "workflowUrl",
              name: t("Workflow Url"),
              value: selectedTrigger.deployment.workflowUrl || t("N/A"),
            },
          ]
        : undefined,
    [t, selectedTrigger],
  );

  return (
    <>
      <div className="flex flex-1 flex-col gap-4 px-6 pt-6 pb-2">
        <div className="flex justify-between">
          <Button size="icon" variant="ghost" onClick={handleBack}>
            <CaretLeftIcon />
          </Button>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={!selectedTrigger}
              onClick={() => setOpenTriggerEditDialog(true)}>
              <PencilLineIcon />
              {t("Update Trigger")}
            </Button>
            <Button
              variant="destructive"
              size="sm"
              onClick={() => setTriggerToBeDeleted(selectedTrigger)}>
              <TrashIcon />
              {t("Delete")}
            </Button>
          </div>
        </div>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
          <DetailsBox title={t("Trigger Details")} content={details} />
        </div>
        {selectedTrigger?.eventSource === "API_DRIVEN" && (
          <div className="mt-4 flex max-w-[1200px] flex-col gap-4 rounded-lg border-muted bg-muted/20 p-6 shadow-sm">
            <p className="flex items-center gap-2 text-base font-semibold">
              <span className="inline-block rounded bg-primary/10 px-2 py-0.5 text-xs font-bold  text-white">
                API
              </span>
              {t("How to Trigger API Driven Event:")}
            </p>
            <div className="space-y-3 text-sm text-muted-foreground">
              <div className="flex items-center gap-2">
                <span className="font-semibold">1. {t("Endpoint:")}</span>
                <span className="rounded border bg-background px-2 py-1 font-mono text-xs break-all">
                  POST {apiUrl}/api/triggers/{selectedTrigger.id}/run
                </span>
                <IconButton
                  size="icon"
                  variant="ghost"
                  className="ml-1"
                  onClick={() =>
                    navigator.clipboard.writeText(
                      `${apiUrl}/api/triggers/${selectedTrigger.id}/run`,
                    )
                  }
                  icon={<CopyIcon />}
                />
              </div>
              <div>
                <span className="font-semibold">2. {t("Auth:")}</span>{" "}
                {t('Add token to "Authorization: Bearer {token}" header')}
              </div>
              <div>
                <span className="font-semibold">
                  3. {t("Custom Variables:")}
                </span>{" "}
                {t('Pass {"with": {"key": "value"}} in body')}
              </div>
              <div>
                <span className="font-semibold">4. {t("Callback:")}</span>{" "}
                {t('Optional "notificationUrl" for status updates')}
              </div>
              <div>
                <span className="font-semibold">5. {t("Response:")}</span>{" "}
                {t("Returns runId, deploymentId, and job status")}
              </div>
              <p className="mt-2 border-t border-muted-foreground/20 pt-2 text-xs italic">
                {t("Copy your auth token - you'll need it for API calls.")}
              </p>
            </div>
          </div>
        )}
      </div>
      {openTriggerEditDialog && selectedTrigger && (
        <TriggerEditDialog
          selectedTrigger={selectedTrigger}
          onDialogClose={() => setOpenTriggerEditDialog(false)}
        />
      )}
    </>
  );
};

export { TriggerDetails };
