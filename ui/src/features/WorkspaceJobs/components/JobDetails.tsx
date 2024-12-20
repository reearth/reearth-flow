import { CaretLeft } from "@phosphor-icons/react";
import { useRouter } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";

import { Button } from "@flow/components";
import { DetailsBox, DetailsBoxContent } from "@flow/features/common";
import { LogsConsole } from "@flow/features/Editor/components/BottomPanel/components";
import { useT } from "@flow/lib/i18n";
import { Job } from "@flow/types";

type Props = {
  selectedJob?: Job;
};

const JobDetails: React.FC<Props> = ({ selectedJob }) => {
  const t = useT();
  const { history } = useRouter();

  const handleBack = useCallback(() => history.go(-1), [history]);

  console.log("selectedJob", selectedJob);

  const details: DetailsBoxContent[] | undefined = useMemo(
    () =>
      selectedJob
        ? [
            {
              id: "id",
              name: t("ID:"),
              value: selectedJob.id,
            },
            {
              id: "project",
              name: t("Project Name:"),
              value: selectedJob.deployment?.projectName ?? t("Unknown"),
            },
            {
              id: "started",
              name: t("Started:"),
              value: selectedJob.startedAt,
            },
            {
              id: "completed",
              name: t("Completed:"),
              value: selectedJob.completedAt ?? t("N/A"),
            },
            {
              id: "ranBy",
              name: t("Ran by:"),
              value: t("Unknown"),
              // value: selectedJob.ranBy ?? t("Unknown"),
            },
            {
              id: "trigger",
              name: t("Trigger:"),
              value: t("Unknown"),
              // value: selectedJob.trigger?.toLocaleUpperCase() ?? t("Unknown"),
            },
            {
              id: "status",
              name: t("Status:"),
              value: selectedJob.status.toLocaleUpperCase(),
            },
          ]
        : undefined,
    [t, selectedJob],
  );

  return (
    selectedJob && (
      <div className="flex flex-1 flex-col gap-4 px-6 pb-2 pt-6">
        <Button size="icon" variant="ghost" onClick={handleBack}>
          <CaretLeft />
        </Button>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
          <DetailsBox title={t("Job details")} content={details} />
          <div className="max-h-[50vh] overflow-auto">
            <LogsConsole />
          </div>
        </div>
      </div>
    )
  );
};

export { JobDetails };
