import { Play } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { runs } from "@flow/mock_data/runsData";
import { useCurrentWorkspace } from "@flow/stores";

const RunsSection: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const navigate = useNavigate();

  const runningRuns = runs.filter((run) => run.status === "running");
  const queuedRuns = runs.filter((run) => run.status === "queued");
  const completeRuns = runs.filter((run) => run.status === "completed");
  const failedRuns = runs.filter((run) => run.status === "failed");

  return (
    <div>
      <div className="flex items-center justify-between gap-2 border-b  p-2">
        <p className="text-lg dark:font-extralight">{t("Runs")}</p>
        <Button
          className="flex h-[30px] gap-2"
          variant="outline"
          onClick={() =>
            navigate({ to: `/workspaces/${currentWorkspace?.id}/runs/new` })
          }>
          <Play weight="thin" />
          <p className="text-xs dark:font-light">{t("New Run")}</p>
        </Button>
      </div>
      <div className="flex flex-col gap-1 p-4">
        <div
          className="-mx-2 -my-1 flex justify-between rounded-md px-2 py-1 hover:cursor-pointer hover:bg-accent"
          onClick={() =>
            navigate({ to: `/workspaces/${currentWorkspace?.id}/runs/running` })
          }>
          <p className="dark:font-thin">{t("Running: ")}</p>
          <p className="dark:font-thin">{runningRuns.length}</p>
        </div>
        <div
          className="-mx-2 -my-1 flex justify-between rounded-md px-2 py-1 hover:cursor-pointer hover:bg-accent"
          onClick={() =>
            navigate({ to: `/workspaces/${currentWorkspace?.id}/runs/queued` })
          }>
          <p className="dark:font-thin">{t("Queued: ")}</p>
          <p className="dark:font-thin">{queuedRuns.length}</p>
        </div>
        <div className="my-1 border-t" />
        <div className="flex flex-col">
          <div
            className="-mx-2 -my-1 flex justify-between rounded-md px-2 py-1 hover:cursor-pointer hover:bg-accent"
            onClick={() =>
              navigate({
                to: `/workspaces/${currentWorkspace?.id}/runs/completed`,
              })
            }>
            <p className="dark:font-thin">{t("Completed (today): ")}</p>
            <p className="dark:font-thin">
              {completeRuns.length + failedRuns.length}
            </p>
          </div>
          {failedRuns.length && (
            <div className="ml-3 mt-1">
              <div className="flex justify-between">
                <p className="text-sm dark:font-thin text-green-500">
                  {t("Successful: ")}
                </p>
                <p className="text-sm dark:font-thin">
                  {completeRuns.filter((r) => r.status === "completed").length}
                </p>
              </div>
              <div className="flex justify-between">
                <p className="text-sm text-red-500">{t("Failed: ")}</p>
                <p className="text-sm">{failedRuns.length}</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export { RunsSection };
