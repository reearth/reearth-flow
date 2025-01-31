import { CaretLeft } from "@phosphor-icons/react";
import { useRouter } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";

import { Button } from "@flow/components";
import { DetailsBox, DetailsBoxContent } from "@flow/features/common";
import { LogsConsole } from "@flow/features/Editor/components/BottomPanel/components";
import { useT } from "@flow/lib/i18n";
import type { Job } from "@flow/types";

type Props = {
  selectedJob?: Job;
};

const JobDetails: React.FC<Props> = ({ selectedJob }) => {
  const t = useT();
  const { history } = useRouter();

  const handleBack = useCallback(() => history.go(-1), [history]); // Go back to previous page

  const details: DetailsBoxContent[] | undefined = useMemo(
    () =>
      selectedJob
        ? [
            {
              id: "id",
              name: t("ID"),
              value: selectedJob.id,
            },
            {
              id: "deploymentId",
              name: t("Deployment ID"),
              value: selectedJob.deploymentId,
            },
            {
              id: "status",
              name: t("Status"),
              value: selectedJob.status,
            },
            {
              id: "startedAt",
              name: t("Started At"),
              value: selectedJob.startedAt,
            },
            {
              id: "completedAt",
              name: t("Completed At"),
              value: selectedJob.completedAt || t("N/A"),
            },
          ]
        : undefined,
    [t, selectedJob],
  );

  return (
    selectedJob && (
      <div className="flex flex-1 flex-col gap-4 px-6 pb-2 pt-6">
        <div className="flex justify-between">
          <Button size="icon" variant="ghost" onClick={handleBack}>
            <CaretLeft />
          </Button>
        </div>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
          <DetailsBox title={t("Job Details")} content={details} />
        </div>
        <div className="mt-6 min-h-0 max-w-[1200px] flex-1">
          <LogsConsole />
        </div>
      </div>
    )
  );
};

export { JobDetails };
