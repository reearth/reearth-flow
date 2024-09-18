import { Play } from "@phosphor-icons/react";
import { useNavigate, useParams } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useState } from "react";

import { Button } from "@flow/components";
import { TopNavigation } from "@flow/features/TopNavigation";
import { useT } from "@flow/lib/i18n";
import { runs as mockRuns } from "@flow/mock_data/runsData";
import { useCurrentWorkspace } from "@flow/stores";
import { Run } from "@flow/types";

import { NewRun, StatusContent, RunDetails } from "./components";

type Status = "running" | "queued" | "completed";

type Tab = Status | "new" | "all";

const Runs: React.FC = () => {
  const t = useT();
  const { tab } = useParams({ strict: false });
  const navigate = useNavigate();
  const [currentWorkspace] = useCurrentWorkspace();

  const [selectedRun, selectRun] = useState<Run>();

  const handleRunSelect = useCallback(
    (run: Run) =>
      navigate({
        to: `/workspaces/${currentWorkspace?.id}/runs/${run.id}`,
      }),
    [currentWorkspace, navigate],
  );

  useEffect(() => {
    if (!isList(tab)) {
      const run = mockRuns.find((run) => run.id === tab);
      selectRun((prev) => (prev?.id !== run?.id ? run : prev));
    }
  }, [selectedRun, currentWorkspace, tab, navigate]);

  const handleTabChange = useCallback(
    (tab: Tab) => {
      selectRun(undefined);
      navigate({ to: `/workspaces/${currentWorkspace?.id}/runs/${tab}` });
    },
    [currentWorkspace, navigate],
  );

  const runs = useMemo(
    () =>
      mockRuns.filter((run) => {
        if (tab === "running") return run.status === "running";
        if (tab === "queued") return run.status === "queued";
        if (tab === "completed")
          return run.status === "completed" || run.status === "failed";
        return true;
      }),
    [tab],
  );

  const statusLabels = useMemo(
    () => ({
      completed: t("Completed runs"),
      running: t("Ongoing runs"),
      queued: t("Queued runs"),
      all: t("All runs"),
    }),
    [t],
  );

  const statuses: { id: Tab; name: string; component?: React.ReactNode }[] =
    useMemo(
      () => [
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
      ],
      [t],
    );

  return (
    <div className="flex h-screen flex-col">
      <TopNavigation />
      <div className="flex flex-1">
        <div className="flex w-[250px] flex-col gap-3 border-r bg-secondary px-2 py-4">
          <div className="flex p-2">
            <p className="flex-1 text-lg dark:font-light">{t("Runs")}</p>
            <Button
              className="gap-1"
              size="sm"
              onClick={() => handleTabChange("new")}>
              <Play />
              <p className="dark:font-extralight">{t("New Run")}</p>
            </Button>
          </div>
          <div className="flex-1">
            <div
              className={`mb-1 rounded-md border-transparent px-2 py-[2px] hover:cursor-pointer hover:bg-accent ${tab === "all" ? "bg-accent text-secondary-foreground" : undefined}`}
              onClick={() => handleTabChange("all")}>
              <p className="dark:font-thin">{t("All")}</p>
            </div>
            <div className="-mx-2 border-b" />
            <div className="flex flex-col gap-1 p-2">
              {statuses.map(({ id, name }) => (
                <div
                  key={id}
                  className={`-mx-2 rounded-md border-l-2 border-transparent px-2 py-[2px] hover:cursor-pointer hover:bg-accent ${tab === id ? "bg-accent text-secondary-foreground" : undefined}`}
                  onClick={() => handleTabChange(id)}>
                  <p className="text-sm dark:font-thin">{name}</p>
                </div>
              ))}
            </div>
          </div>
        </div>
        <div className="flex flex-1 flex-col">
          <div className="flex-1">
            {tab === "new" ? (
              <NewRun />
            ) : isList(tab) ? (
              <StatusContent
                label={statusLabels[tab as Status]}
                runs={runs}
                selectedRun={selectedRun}
                onRunSelect={handleRunSelect}
              />
            ) : (
              <RunDetails selectedRun={selectedRun} />
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export { Runs };

function isList(value: string) {
  return !!(
    value === "running" ||
    value === "queued" ||
    value === "completed" ||
    value === "all"
  );
}
