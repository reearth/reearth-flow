import { Play } from "@phosphor-icons/react";
import { useNavigate, useParams } from "@tanstack/react-router";
import { useState } from "react";

import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { runs as mockRuns } from "@flow/mock_data/runsData";
import { Run } from "@flow/types";

import { TopNavigation } from "../TopNavigation";

import { ManualRun } from "./components";
import { StatusContent } from "./components/StatusContent";

type Status = "running" | "queued" | "completed";

type Tab = Status | "manual";

const Runs: React.FC = () => {
  const t = useT();
  const { tab } = useParams({ strict: false });
  const navigate = useNavigate();
  const { workspaceId } = useParams({ strict: false });

  const [selectedRun, selectRun] = useState<Run>();

  const handleTabChange = (tab: Tab) => {
    selectRun(undefined);
    navigate({ to: `/workspace/${workspaceId}/runs/${tab}` });
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

  const statuses: { id: Tab; name: string; component?: React.ReactNode }[] = [
    {
      id: "completed",
      name: t("Completed"),
    },
    {
      id: "running",
      name: t("Running"),
    },
    {
      id: "queued",
      name: t("Queued"),
    },
  ];

  return (
    <div className="flex flex-col bg-zinc-800 text-zinc-300 h-[100vh]">
      <TopNavigation />
      <div className="flex flex-1">
        <div className="flex flex-col gap-3 px-2 py-4 bg-zinc-900/50 border-r border-zinc-700 w-[250px]">
          <div className="flex p-2">
            <Button
              className="flex-1 gap-2"
              size="sm"
              // variant="ghost"
              onClick={() => handleTabChange("manual")}>
              <Play />
              <p className="font-extralight">{t("Manual Run")}</p>
            </Button>
          </div>
          <div className="flex-1">
            <p className="font-thin text-lg border-b border-zinc-700 py-2 px-4">{t("Status")}</p>
            <div className="flex flex-col gap-4 p-4">
              {statuses.map(({ id, name }) => (
                <div
                  key={id}
                  className={`flex justify-between py-1 -my-1 px-2 -mx-2 rounded-md border-l-2 border-transparent hover:bg-zinc-700/50 hover:cursor-pointer ${tab === id ? "bg-zinc-700/50 text-white border-red-800/50" : undefined}`}
                  onClick={() => handleTabChange(id)}>
                  <p className="font-thin">{name}</p>
                </div>
              ))}
            </div>
          </div>
        </div>
        <div className="flex flex-col flex-1">
          <div className="flex-1">
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
