import {
  CaretLeftIcon,
  CopyIcon,
  PencilLineIcon,
  QuestionIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { useRouter } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";

import {
  Button,
  Dialog,
  IconButton,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@flow/components";
import { config } from "@flow/config";
import type { DetailsBoxContent } from "@flow/features/common";
import { DetailsBox, toWorkflowVariablesDetail } from "@flow/features/common";
import ConfirmationDialog from "@flow/features/ConfirmationDialog";
import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useTrigger } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import type { Trigger, TriggerRefusal, TriggerRefusalCode } from "@flow/types";
import {
  formatTimestamp,
  getEventSourceLabel,
  getTimeIntervalLabel,
} from "@flow/utils";
import { copyToClipboard } from "@flow/utils/copyToClipboard";

import { isRetrying } from "../lastRefusal";

import { TriggerNewAuthToken } from "./TriggerAuthToken";
import { TriggerEditDialog } from "./TriggerEditDialog";
import { triggerRunValues } from "./TriggerWorkflowVariables/triggerVariables";

type Props = {
  selectedTrigger?: Trigger;
  readonly?: boolean;
  setTriggerToBeDeleted: (trigger?: Trigger) => void;
};

const TriggerDetails: React.FC<Props> = ({
  selectedTrigger,
  readonly,
  setTriggerToBeDeleted,
}) => {
  const t = useT();
  const { toast } = useToast();
  const { useUpdateTrigger, regenerateTriggerToken } = useTrigger();
  const { history } = useRouter();
  const [openTriggerEditDialog, setOpenTriggerEditDialog] = useState(false);
  const [openRegenerateTokenDialog, setOpenRegenerateTokenDialog] =
    useState(false);
  // A re-generated token, shown once.
  const [issuedAuthToken, setIssuedAuthToken] = useState<string | undefined>(
    undefined,
  );

  const handleRegenerateToken = useCallback(async () => {
    setOpenRegenerateTokenDialog(false);
    if (!selectedTrigger) return;
    const { authToken } = await regenerateTriggerToken(selectedTrigger.id);
    setIssuedAuthToken(authToken);
  }, [selectedTrigger, regenerateTriggerToken]);
  const updateTrigger = useUpdateTrigger;

  const [updatedIsTriggerEnabled, setUpdatedIsTriggerEnabled] = useState(
    selectedTrigger?.enabled,
  );

  const handleTriggerEnableChange = useCallback(
    async (checked: boolean) => {
      if (!selectedTrigger) return;
      setUpdatedIsTriggerEnabled(checked);
      await updateTrigger(selectedTrigger.id, checked);
    },
    [selectedTrigger, updateTrigger],
  );

  const handleBack = useCallback(() => history.go(-1), [history]); // Go back to previous page

  // Why the last scheduled run did not start, and whether it is still being
  // retried.
  const lastRefusalText = useCallback(
    (refusal: TriggerRefusal) => {
      const reasons: Record<TriggerRefusalCode, string> = {
        DEPLOYMENT_NOT_FOUND: t("The deployment no longer exists."),
        TRIGGER_DISABLED: t("The trigger is disabled."),
        INVALID_VARIABLES: t(
          "A required workflow variable has no value, or a value does not fit its type.",
        ),
        NOT_ALLOWED: t("The run was not allowed."),
        CONCURRENT_JOB_LIMIT: t(
          "The workspace was running as many jobs as its plan allows.",
        ),
        SERVER_ERROR: t("The server could not start the run."),
      };
      const reason = reasons[refusal.code];
      return refusal.retryUntil && isRetrying(refusal)
        ? t("Retrying until {{time}}: {{reason}}", {
            time: formatTimestamp(refusal.retryUntil),
            reason,
          })
        : t("Skipped at {{time}}: {{reason}}", {
            time: formatTimestamp(refusal.refusedAt),
            reason,
          });
    },
    [t],
  );
  const apiUrl = config().api || window.location.origin;
  const details: DetailsBoxContent[] | undefined = useMemo(
    () =>
      selectedTrigger
        ? [
            {
              id: "id",
              name: t("ID"),
              value: selectedTrigger.id ?? t("Unknown or deleted trigger"),
            },
            {
              id: "triggerId",
              name: t("Trigger Description"),
              value: selectedTrigger.description ?? t("N/A"),
            },
            {
              id: "deploymentId",
              name: t("Deployment Id"),
              value: selectedTrigger.deploymentId ?? t("N/A"),
            },
            {
              id: "projectName",
              name: t("Project Name"),
              value:
                selectedTrigger.deployment.projectName ??
                t("Unknown or deleted project"),
            },
            {
              id: "deploymentDescription",
              name: t("Deployment Description"),
              value: selectedTrigger.deployment.description
                ? selectedTrigger.variables
                  ? `${selectedTrigger.deployment.description} ${t("[defaults overridden]")}`
                  : selectedTrigger.deployment.description
                : t("N/A"),
            },
            {
              id: "eventSource",
              name: t("Event Source"),
              value:
                getEventSourceLabel(selectedTrigger.eventSource, t) ?? t("N/A"),
            },
            ...(selectedTrigger.eventSource === "API_DRIVEN"
              ? [
                  {
                    id: "authToken",
                    name: t("Auth Token"),
                    value: selectedTrigger.authTokenHint
                      ? `••••${selectedTrigger.authTokenHint}`
                      : t("Set by hand"),
                  },
                ]
              : []),
            ...(selectedTrigger.eventSource === "TIME_DRIVEN"
              ? [
                  {
                    id: "timeInterval",
                    name: t("Time Interval"),
                    value:
                      getTimeIntervalLabel(selectedTrigger.timeInterval, t) ??
                      t("N/A"),
                  },
                ]
              : []),
            {
              id: "lastTriggered",
              name: t("Last Triggered"),
              value: selectedTrigger.lastTriggered
                ? formatTimestamp(selectedTrigger.lastTriggered)
                : t("Never"),
            },
            ...(selectedTrigger.lastRefusal
              ? [
                  {
                    id: "lastRefusal",
                    name: t("Last Scheduled Run"),
                    value: lastRefusalText(selectedTrigger.lastRefusal),
                  },
                ]
              : []),
            {
              id: "createdAt",
              name: t("Created At"),
              value: formatTimestamp(selectedTrigger.createdAt) ?? t("Never"),
            },
            {
              id: "updatedAt",
              name: t("Updated At"),
              value: formatTimestamp(selectedTrigger.updatedAt) ?? t("Never"),
            },
            {
              id: "workflowUrl",
              name: t("Workflow Url"),
              value: selectedTrigger.deployment.workflowUrl ?? t("N/A"),
            },
            ...toWorkflowVariablesDetail(
              triggerRunValues(
                selectedTrigger.deployment.parameters,
                selectedTrigger.variables,
              ),
              { title: t("Workflow Variables"), empty: t("N/A") },
            ),
          ]
        : undefined,
    [t, selectedTrigger, lastRefusalText],
  );

  const handleCopyToClipboard = useCallback(
    (url: string) => {
      copyToClipboard(url);
      toast({
        title: t("Copied to clipboard"),
        description: t("URL copied to clipboard"),
      });
    },
    [t, toast],
  );

  return (
    <>
      <div className="flex flex-1 flex-col gap-4 overflow-y-auto px-6 pt-6 pb-2">
        <div className="flex justify-between">
          <Button size="icon" variant="ghost" onClick={handleBack}>
            <CaretLeftIcon />
          </Button>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={readonly}
              onClick={() => setOpenTriggerEditDialog(true)}>
              <PencilLineIcon />
              {t("Update Trigger")}
            </Button>
            <Button
              variant="destructive"
              size="sm"
              disabled={readonly}
              onClick={() => setTriggerToBeDeleted(selectedTrigger)}>
              <TrashIcon />
              {t("Delete")}
            </Button>
          </div>
        </div>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
          <DetailsBox
            title={t("Trigger Details")}
            content={details}
            toggle
            toggleValue={updatedIsTriggerEnabled}
            disableToggle={readonly}
            onToggleChange={handleTriggerEnableChange}
          />
        </div>
        {selectedTrigger?.eventSource === "API_DRIVEN" && (
          <div className="mt-2 flex max-w-[1200px] flex-col gap-4 rounded-lg border-muted bg-muted/20 p-5 shadow-sm">
            <p className="flex items-center gap-2 text-base font-semibold">
              <span className="inline-block rounded bg-border px-2 py-0.5 text-xs font-bold  text-white">
                API
              </span>
              {t("How to Trigger API Driven Event:")}
            </p>
            <div className="flex flex-nowrap items-center gap-2">
              <span className="font-semibold">1. {t("Endpoint:")}</span>
              <div className="max-w-[200px] overflow-x-auto overflow-y-hidden p-1">
                <span className="rounded border bg-background px-2 py-1 font-mono text-xs whitespace-nowrap">
                  POST {apiUrl}/api/triggers/{selectedTrigger.id}/run
                </span>
              </div>
              <IconButton
                size="icon"
                variant="ghost"
                className="ml-1 shrink-0"
                onClick={() =>
                  handleCopyToClipboard(
                    `${apiUrl}/api/triggers/${selectedTrigger.id}/run`,
                  )
                }
                icon={<CopyIcon />}
              />
            </div>
            <div>
              <span className="font-semibold">2. {t("Auth:")}</span>{" "}
              {t(
                'Add token to "Authorization: Bearer <your_auth_token>" header',
              )}
              <Button
                variant="outline"
                size="sm"
                className="ml-2"
                disabled={readonly}
                onClick={() => setOpenRegenerateTokenDialog(true)}>
                {t("Re-generate")}
              </Button>
              {!selectedTrigger.authTokenHint && (
                <p className="mt-1 text-xs text-warning">
                  {t(
                    "This token was set by hand before Flow generated tokens. Re-generate it to get a strong one.",
                  )}
                </p>
              )}
            </div>
            <div className="flex flex-wrap items-center">
              <span className="font-semibold">3. {t("Custom Variables")}</span>

              <Tooltip>
                <TooltipTrigger
                  render={
                    <div className="cursor-pointer p-1">
                      <QuestionIcon className="h-4 w-4" weight="thin" />
                    </div>
                  }
                />
                <TooltipContent side="top" align="end" className="bg-primary">
                  <div className="max-w-[300px] text-xs text-muted-foreground">
                    {t(
                      'Pass {"with": {"key": "value"}} in body to inject dynamic parameters into workflow execution. These variables override/supplement default workflow values and are accessible in actions.',
                    )}
                  </div>
                </TooltipContent>
              </Tooltip>

              <span className="mx-1">:</span>

              <span>{t('Pass {"with": {"key": "value"}} in body')}</span>
            </div>

            <div>
              <span className="font-semibold">4. {t("Callback:")}</span>{" "}
              {t('Optional "notificationUrl" for status updates')}
            </div>
            <div>
              <span className="font-semibold">5. {t("Response:")}</span>{" "}
              {t("Returns runId, deploymentId, and job status")}
            </div>
          </div>
        )}
      </div>
      <ConfirmationDialog
        title={t("Re-generate The Auth Token?")}
        description={t(
          "If you re-generate the auth token, the previous token will become invalid, and this action cannot be undone. Are you sure you want to proceed?",
        )}
        isOpen={openRegenerateTokenDialog}
        onClose={() => setOpenRegenerateTokenDialog(false)}
        onConfirm={handleRegenerateToken}
      />
      {issuedAuthToken && (
        <Dialog open={true} onOpenChange={() => setIssuedAuthToken(undefined)}>
          <TriggerNewAuthToken
            authToken={issuedAuthToken}
            onDone={() => setIssuedAuthToken(undefined)}
          />
        </Dialog>
      )}
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
