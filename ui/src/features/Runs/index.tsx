import { Play } from "@phosphor-icons/react";
import { useNavigate, useParams } from "@tanstack/react-router";
import { useState } from "react";

import { Button, FlowLogo } from "@flow/components";
import { runs as mockRuns } from "@flow/mock_data/runsData";
import { useT } from "@flow/providers";
import { useCurrentWorkspace } from "@flow/stores";
import { Run } from "@flow/types";

import { UserNavigation, WorkspaceNavigation } from "../Dashboard/components/Nav/components";

import { ManualRun } from "./components";
import { StatusContent } from "./components/StatusContent";

type Status = "running" | "queued" | "completed";

type Tab = Status | "manual";

const Runs: React.FC = () => {
  const t = useT();
  const { tab } = useParams({ strict: false });
  const navigate = useNavigate();
  const [currentWorkspace] = useCurrentWorkspace();

  const [selectedRun, selectRun] = useState<Run>();

  const handleTabChange = (tab: Tab) => {
    selectRun(undefined);
    navigate({ to: `/workspace/${currentWorkspace?.id}/runs/${tab}` });
  };

  const runs = mockRuns.filter(run => {
    if (tab === "running") return run.status === "running";
    if (tab === "queued") return run.status === "queued";
    if (tab === "completed") return run.status === "completed" || run.status === "failed";
    return true;
  });

  const statusLabels = {
    completed: t("Completed"),
    running: t("Running"),
    queued: t("Queued"),
  };

  return (
    <div className="flex flex-col bg-zinc-800 text-zinc-300 h-[100vh]">
      <div className={`bg-zinc-900/50 border-b border-zinc-700`}>
        <div className="relative flex justify-between items-center gap-4 h-14 px-4">
          <div
            className="bg-red-800/50 p-2 rounded cursor-pointer z-10"
            onClick={() => navigate({ to: `/workspace/${currentWorkspace?.id}` })}>
            <FlowLogo className="h-5 w-5" />
          </div>
          <div id="dashboard-middle" className="absolute left-0 right-0 flex justify-center">
            <div className="flex justify-center gap-4 max-w-[40vw]">
              <WorkspaceNavigation />
            </div>
          </div>
          <div id="dashboard-right" className="flex items-center z-10">
            <UserNavigation />
          </div>
        </div>
      </div>
      <div className="flex flex-1 m-[8px] gap-[8px]">
        <div className="flex flex-col gap-[8px]">
          <div className="flex bg-zinc-900/50 border border-zinc-700 px-4 py-2 rounded-lg">
            <Button
              className="flex-1 gap-2"
              size="sm"
              variant="ghost"
              onClick={() => handleTabChange("manual")}>
              <Play />
              <p className="font-extralight">{t("Manual Run")}</p>
            </Button>
          </div>
          <div className="flex-1 w-[200px] bg-zinc-900/50 border border-zinc-700 rounded-lg">
            <p className="font-thin text-lg border-b border-zinc-700 py-2 px-4">{t("Status")}</p>
            <div className="flex flex-col gap-2 p-4">
              <div
                className={`flex justify-between py-1 -my-1 px-2 -mx-2 rounded-md hover:bg-zinc-700/50 hover:cursor-pointer ${tab === "completed" ? "bg-zinc-700/50 text-white" : undefined}`}
                onClick={() => handleTabChange("completed")}>
                <p className="font-thin">{t("Completed")}</p>
              </div>
              <div
                className={`flex justify-between py-1 -my-1 px-2 -mx-2 rounded-md hover:bg-zinc-700/50 hover:cursor-pointer ${tab === "running" ? "bg-zinc-700/50 text-white" : undefined}`}
                onClick={() => handleTabChange("running")}>
                <p className="font-thin">{t("Running")}</p>
              </div>
              <div
                className={`flex justify-between py-1 -my-1 px-2 -mx-2 rounded-md hover:bg-zinc-700/50 hover:cursor-pointer ${tab === "queued" ? "bg-zinc-700/50 text-white" : undefined}`}
                onClick={() => handleTabChange("queued")}>
                <p className="font-thin">{t("Queued")}</p>
              </div>
            </div>
          </div>
        </div>
        <div className="flex flex-col flex-1">
          <div className="flex-1 bg-zinc-900/50 border border-zinc-700 rounded-lg">
            {tab === "manual" ? (
              <ManualRun />
            ) : (
              <StatusContent
                label={statusLabels[tab as Status]}
                runs={runs}
                selectedRun={selectedRun}
                onRunSelect={selectRun}
              />
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export { Runs };
