import { CopyIcon, QuestionIcon } from "@phosphor-icons/react";

import {
  DialogContent,
  DialogContentWrapper,
  DialogTitle,
  IconButton,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@flow/components";
import { config } from "@flow/config";
import { useT } from "@flow/lib/i18n";
import { Trigger } from "@flow/types";

type Props = {
  createdTrigger: Trigger;
  onCopyToClipboard: (text: string) => void;
};

const TriggerApiDrivenDetails: React.FC<Props> = ({
  createdTrigger,
  onCopyToClipboard,
}) => {
  const t = useT();
  const apiUrl = config().api || window.location.origin;
  return (
    <DialogContent size="sm">
      <DialogTitle>{t("How to Trigger API Driven Event:")}</DialogTitle>
      <DialogContentWrapper className="overflow-hidden">
        <div className="space-y-3 text-sm text-muted-foreground">
          <div className="flex flex-nowrap items-center gap-2">
            <span className="font-semibold">1. {t("Endpoint:")}</span>
            <div className="max-w-[200px] overflow-x-auto overflow-y-hidden p-1">
              <span className="rounded border bg-background px-2 py-1 font-mono text-xs whitespace-nowrap">
                POST {apiUrl}/api/triggers/{createdTrigger.id}/run
              </span>
            </div>
            <IconButton
              size="icon"
              variant="ghost"
              className="ml-1 shrink-0"
              onClick={() =>
                onCopyToClipboard(
                  `${apiUrl}/api/triggers/${createdTrigger.id}/run`,
                )
              }
              icon={<CopyIcon />}
            />
          </div>
          <div>
            <span className="font-semibold">2. {t("Auth:")}</span>{" "}
            {t(
              `Add token to "Authorization: Bearer ${createdTrigger.authToken}" header`,
            )}
          </div>
          <div className="flex flex-wrap items-center">
            <span className="font-semibold">3. {t("Custom Variables")}</span>

            <Tooltip>
              <TooltipTrigger asChild>
                <div className="cursor-pointer p-1">
                  <QuestionIcon className="h-4 w-4" weight="thin" />
                </div>
              </TooltipTrigger>
              <TooltipContent side="top" align="end" className="bg-primary">
                <div className="max-w-[300px] text-xs text-muted-foreground">
                  {t(
                    'Pass {"with": {"key": "value"}} in body to inject dynamic parameters into workflow execution. These variables override/supplement default workflow values and are accessible in actions.',
                  )}
                </div>
              </TooltipContent>
            </Tooltip>

            <span className="mx-1">:</span>

            <span className="text-muted-foreground">
              {t('Pass {"with": {"key": "value"}} in body')}
            </span>
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

        <p className="mt-2 border-t border-muted-foreground/20 pt-2 text-xs">
          {t(
            "You can review these details any time on the trigger's details page",
          )}
        </p>
      </DialogContentWrapper>
    </DialogContent>
  );
};

export { TriggerApiDrivenDetails };
