import { CaretLeftIcon, XCircleIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { Button } from "@flow/components";
import { DetailsBox, OutputArtifacts } from "@flow/features/common";
import ConfirmationDialog from "@flow/features/ConfirmationDialog";
import LogsConsole from "@flow/features/LogsConsole";
import { useJobSubscriptionsSetup } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";

import useHooks from "./hooks";

type Props = {
  jobId: string;
  accessToken: string;
};

const JobDetails: React.FC<Props> = ({ jobId, accessToken }) => {
  const t = useT();

  useJobSubscriptionsSetup(accessToken, jobId);

  const [isConfirmingCancel, setIsConfirmingCancel] = useState(false);

  const { job, details, outputFiles, jobStatus, handleBack, handleCancelJob } =
    useHooks({
      jobId,
    });

  return (
    job && (
      <div className="flex flex-1 flex-col gap-4 overflow-y-auto px-6 pt-6 pb-2">
        <div className="flex justify-between">
          <Button size="icon" variant="ghost" onClick={handleBack}>
            <CaretLeftIcon />
          </Button>
          {(jobStatus === "queued" || jobStatus === "running") && (
            <Button
              variant="destructive"
              size="sm"
              onClick={() => setIsConfirmingCancel(true)}>
              <XCircleIcon />
              {t("Cancel Job")}
            </Button>
          )}
        </div>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col">
          <DetailsBox
            collapsible
            title={t("Job Details")}
            content={details}
            status={jobStatus || job.status}
          />
        </div>
        {outputFiles.length > 0 && (
          <div className="flex max-w-[1200px] flex-col">
            <OutputArtifacts files={outputFiles} archiveName={`job_${jobId}`} />
          </div>
        )}
        <div className="flex items-center">
          <h2 className="text-lg">{t("Log")}</h2>
        </div>
        {/* Fills the space left, but keeps a usable height when the boxes
            above are open, so the page scrolls instead of the log vanishing. */}
        <div className="min-h-[400px] max-w-[1200px] flex-1">
          <LogsConsole jobId={job.id} />
        </div>
        <ConfirmationDialog
          title={t("Cancel this job?")}
          description={t(
            "The job stops where it is and cannot be resumed. Outputs it has not written yet are not created.",
          )}
          isOpen={
            isConfirmingCancel &&
            (jobStatus === "queued" || jobStatus === "running")
          }
          confirmLabel={t("Cancel Job")}
          cancelLabel={t("Keep Running")}
          onClose={() => setIsConfirmingCancel(false)}
          onConfirm={() => {
            setIsConfirmingCancel(false);
            handleCancelJob();
          }}
        />
      </div>
    )
  );
};

export { JobDetails };
