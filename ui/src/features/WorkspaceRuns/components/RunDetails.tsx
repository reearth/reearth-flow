import { CaretLeft } from "@phosphor-icons/react";
import { useRouter } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";

import { Button } from "@flow/components";
import { DetailsBox, DetailsBoxContent } from "@flow/features/common";
import { LogsConsole } from "@flow/features/Editor/components/BottomPanel/components";
import { useT } from "@flow/lib/i18n";
import { Run } from "@flow/types";

type Props = {
  selectedRun?: Run;
};

const RunDetails: React.FC<Props> = ({ selectedRun }) => {
  const t = useT();
  const { history } = useRouter();

  const handleBack = useCallback(() => history.go(-1), [history]);

  const details: DetailsBoxContent[] | undefined = useMemo(
    () =>
      selectedRun
        ? [
            {
              id: "id",
              name: t("ID:"),
              value: selectedRun.id,
            },
            {
              id: "project",
              name: t("Project Name:"),
              value: selectedRun.project.name,
            },
            {
              id: "started",
              name: t("Started:"),
              value: selectedRun.startedAt,
            },
            {
              id: "completed",
              name: t("Completed:"),
              value: selectedRun.completedAt ?? t("N/A"),
            },
            {
              id: "ranBy",
              name: t("Ran by:"),
              value: selectedRun.ranBy ?? t("Unknown"),
            },
            {
              id: "trigger",
              name: t("Trigger:"),
              value: selectedRun.trigger?.toLocaleUpperCase() ?? t("Unknown"),
            },
            {
              id: "status",
              name: t("Status:"),
              value: selectedRun.status.toLocaleUpperCase(),
            },
          ]
        : undefined,
    [t, selectedRun],
  );

  return (
    selectedRun && (
      <div className="flex flex-1 flex-col gap-4 px-6 pb-2 pt-6">
        <Button size="icon" variant="ghost" onClick={handleBack}>
          <CaretLeft />
        </Button>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
          <DetailsBox title={t("Run details")} content={details} />
          <div className="max-h-[50vh] overflow-auto">
            <LogsConsole />
          </div>
        </div>
      </div>
    )
  );
};

export { RunDetails };
