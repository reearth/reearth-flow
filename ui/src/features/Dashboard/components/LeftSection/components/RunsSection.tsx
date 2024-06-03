import { Play } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import { ButtonWithTooltip } from "@flow/components";
import { runs } from "@flow/mock_data/runsData";
import { useT } from "@flow/providers";
import { useCurrentWorkspace } from "@flow/stores";

const RunsSection: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const navigate = useNavigate();

  const runningRuns = runs.filter(run => run.status === "running");
  const queuedRuns = runs.filter(run => run.status === "queued");
  const completeRuns = runs.filter(run => run.status === "completed");
  const failedRuns = runs.filter(run => run.status === "failed");

  return (
    <div className="bg-zinc-900/50 border border-zinc-700 rounded-lg">
      <div className="flex gap-2  justify-between items-center border-b border-zinc-700 py-2 px-4">
        <p className="text-lg font-extralight">{t("Runs")}</p>
        <ButtonWithTooltip
          className="flex gap-1 font-extralight bg-zinc-800 text-zinc-400 hover:bg-zinc-700 hover:text-zinc-300"
          variant="outline"
          tooltipText="Run a new flow"
          tooltipPosition="left"
          tooltipOffset={5}
          onClick={() => navigate({ to: `/workspace/${currentWorkspace?.id}/runs/manual` })}>
          <Play />
        </ButtonWithTooltip>
      </div>
      <div className="flex flex-col gap-1 p-4">
        <div
          className="flex justify-between py-1 -my-1 px-2 -mx-2 rounded-md hover:bg-zinc-800 hover:cursor-pointer"
          onClick={() => navigate({ to: `/workspace/${currentWorkspace?.id}/runs/running` })}>
          <p className="font-thin">{t("Running: ")}</p>
          <p className="font-thin">{runningRuns.length}</p>
        </div>
        <div
          className="flex justify-between py-1 -my-1 px-2 -mx-2 rounded-md hover:bg-zinc-800 hover:cursor-pointer"
          onClick={() => navigate({ to: `/workspace/${currentWorkspace?.id}/runs/queued` })}>
          <p className="font-thin">{t("Queued: ")}</p>
          <p className="font-thin">{queuedRuns.length}</p>
        </div>
        <div className="border-t border-zinc-700 my-1" />
        <div className="flex flex-col">
          <div
            className="flex justify-between py-1 -my-1 px-2 -mx-2 rounded-md hover:bg-zinc-800 hover:cursor-pointer"
            onClick={() => navigate({ to: `/workspace/${currentWorkspace?.id}/runs/completed` })}>
            <p className="font-thin">{t("Completed (today): ")}</p>
            <p className="font-thin">{completeRuns.length + failedRuns.length}</p>
          </div>
          {failedRuns.length && (
            <div className="ml-3 mt-1">
              <div className="flex justify-between">
                <p className="font-thin text-sm text-green-500">{t("Successful: ")}</p>
                <p className="font-thin text-sm">
                  {completeRuns.filter(r => r.status === "completed").length}
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
