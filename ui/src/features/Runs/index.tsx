import { Play } from "@phosphor-icons/react";
import { useNavigate, useParams } from "@tanstack/react-router";
import { useState } from "react";

import { Button } from "@flow/components";
import { TopNavigation } from "@flow/features/TopNavigation";
import { useT } from "@flow/lib/i18n";
import { runs as mockRuns } from "@flow/mock_data/runsData";
import { useCurrentWorkspace } from "@flow/stores";
import { Run } from "@flow/types";

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
    <div className="flex h-screen flex-col">
      <TopNavigation />
      <div className="flex flex-1">
        <div className="flex w-[250px] flex-col gap-3 border-r bg-secondary px-2 py-4">
          <div className="flex p-2">
            <Button
              className="flex-1 gap-2"
              size="sm"
              variant="outline"
              onClick={() => handleTabChange("manual")}>
              <Play />
              <p className="font-extralight">{t("Manual Run")}</p>
            </Button>
          </div>
          <div className="flex-1">
            <p className="border-b  px-4 py-2 text-lg font-thin">{t("Status")}</p>
            <div className="flex flex-col gap-4 p-4">
              {statuses.map(({ id, name }) => (
                <div
                  key={id}
                  className={`-mx-2 -my-1 flex justify-between rounded-md border-l-2 border-transparent px-2 py-1 hover:cursor-pointer hover:bg-accent ${tab === id ? "bg-accent text-accent-foreground" : undefined}`}
                  onClick={() => handleTabChange(id)}>
                  <p className="font-thin">{name}</p>
                </div>
              ))}
            </div>
          </div>
        </div>
        <div className="flex flex-1 flex-col">
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
