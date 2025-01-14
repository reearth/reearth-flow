import { Play } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import { Button } from "@flow/components";
import type { RouteOption } from "@flow/features/WorkspaceLeftPanel";
import { useT } from "@flow/lib/i18n";
import { jobs } from "@flow/mock_data/jobsData";
import { useCurrentWorkspace } from "@flow/stores";

type Props = {
  route?: RouteOption;
};

// TODO: Update and use JobsSection again in WorkspaceLeftPanel, or remove it if it's not needed anymore
const JobsSection: React.FC<Props> = ({ route }) => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const navigate = useNavigate();

  const runningJobs = jobs.filter((job) => job.status === "running");
  const queuedJobs = jobs.filter((job) => job.status === "queued");
  const completeJobs = jobs.filter((job) => job.status === "completed");
  const failedJobs = jobs.filter((job) => job.status === "failed");

  return (
    <div>
      <div className="flex items-center justify-between gap-2 p-2">
        <p
          className="-m-1 cursor-pointer rounded p-1 text-lg hover:bg-accent dark:font-extralight"
          onClick={() =>
            navigate({ to: `/workspaces/${currentWorkspace?.id}/jobs/all` })
          }>
          {t("Jobs")}
        </p>
        <Button
          className="flex h-[30px] gap-2"
          variant="outline"
          onClick={() =>
            navigate({ to: `/workspaces/${currentWorkspace?.id}/jobs/new` })
          }>
          <Play weight="thin" />
          <p className="text-xs dark:font-light">{t("New Job")}</p>
        </Button>
      </div>
      <div className="m-1 flex flex-col gap-1 rounded border bg-zinc-600/20 p-2">
        <div
          className={`-m-1 flex justify-between rounded-md p-1 hover:cursor-pointer ${route === "running" && "bg-accent"} hover:bg-accent`}
          onClick={() =>
            navigate({ to: `/workspaces/${currentWorkspace?.id}/jobs/running` })
          }>
          <p className="dark:font-thin">{t("Running: ")}</p>
          <p className="dark:font-thin">{runningJobs.length}</p>
        </div>
        <div
          className={`-m-1 flex justify-between rounded-md p-1 hover:cursor-pointer ${route === "queued" && "bg-accent"} hover:bg-accent`}
          onClick={() =>
            navigate({ to: `/workspaces/${currentWorkspace?.id}/jobs/queued` })
          }>
          <p className="dark:font-thin">{t("Queued: ")}</p>
          <p className="dark:font-thin">{queuedJobs.length}</p>
        </div>
        <div className="my-1 border-t" />
        <div className="flex flex-col">
          <div
            className={`-m-1 flex justify-between rounded-md p-1 hover:cursor-pointer ${route === "completed" && "bg-accent"} hover:bg-accent`}
            onClick={() =>
              navigate({
                to: `/workspaces/${currentWorkspace?.id}/jobs/completed`,
              })
            }>
            <p className="dark:font-thin">{t("Completed (today): ")}</p>
            <p className="dark:font-thin">
              {completeJobs.length + failedJobs.length}
            </p>
          </div>
          {failedJobs.length && (
            <div className="ml-3 mt-1">
              <div className="flex justify-between">
                <p className="text-sm text-green-500 dark:font-thin">
                  {t("Successful: ")}
                </p>
                <p className="text-sm dark:font-thin">
                  {completeJobs.filter((j) => j.status === "completed").length}
                </p>
              </div>
              <div className="flex justify-between">
                <p className="text-sm text-red-500">{t("Failed: ")}</p>
                <p className="text-sm">{failedJobs.length}</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export { JobsSection };
