import { CaretLeftIcon, XCircleIcon } from "@phosphor-icons/react";

import { Button } from "@flow/components";
import { DetailsBox, OutputArtifacts } from "@flow/features/common";
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

  const { job, details, outputFiles, jobStatus, handleBack, handleCancelJob } =
    useHooks({
      jobId,
    });

  return (
    job && (
      <div className="flex flex-1 flex-col gap-4 px-6 pt-6 pb-2">
        <div className="flex justify-between">
          <Button size="icon" variant="ghost" onClick={handleBack}>
            <CaretLeftIcon />
          </Button>
          {(jobStatus === "queued" || jobStatus === "running") && (
            <Button variant="destructive" size="sm" onClick={handleCancelJob}>
              <XCircleIcon />
              {t("Cancel Job")}
            </Button>
          )}
        </div>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col">
          <DetailsBox collapsible title={t("Job Details")} content={details} />
        </div>
        {outputFiles.length > 0 && (
          <div className="flex max-w-[1200px] flex-col">
            <OutputArtifacts files={outputFiles} archiveName={`job_${jobId}`} />
          </div>
        )}
        <div className="flex items-center">
          <h2 className="text-lg">{t("Log")}</h2>
        </div>
        <div className="min-h-0 max-w-[1200px] flex-1">
          <LogsConsole jobId={job.id} />
        </div>
      </div>
    )
  );
};

export { JobDetails };
