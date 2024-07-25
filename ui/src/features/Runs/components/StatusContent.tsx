import { CaretDown, CaretUp } from "@phosphor-icons/react";
import { useState } from "react";

import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Run } from "@flow/types";

import { LogConsole } from "../../Editor/components/BottomPanel/components";

import { RunsTable } from "./RunsTable";

type Props = {
  label: string;
  runs: Run[];
  selectedRun?: Run;
  onRunSelect: ((run: Run) => void) | undefined;
};

const StatusContent: React.FC<Props> = ({ label, runs, selectedRun, onRunSelect }) => {
  const [showLogs, setShowLogs] = useState(false);
  const t = useT();

  return (
    <div className="flex-1 p-8">
      <div className="flex items-center gap-2 text-lg font-extralight">
        <p>{t("Runs Manager")}</p>
        <p className="text-sm font-thin text-zinc-400">({label})</p>
      </div>
      <div className="mt-4 flex max-w-[1200px] flex-col gap-6">
        <div className="max-h-[30vh] overflow-auto rounded-md px-2">
          <RunsTable runs={runs} selectedRun={selectedRun} onRunSelect={onRunSelect} />
        </div>
        {selectedRun && (
          <div className="mx-4 rounded-md border border-zinc-700 font-thin text-zinc-300">
            <div className="border-b border-zinc-700 px-4 py-2">
              <p className="text-xl">{t("Run details")}</p>
            </div>
            <div className="flex max-h-[45vh] flex-col gap-2 p-4">
              <p>
                {t("ID:")} <span className="font-normal">{selectedRun.id}</span>
              </p>
              <p>
                {t("Project Name:")} <span className="font-normal">{selectedRun.project.name}</span>
              </p>
              <div className="flex gap-5">
                <p>
                  {t("Started:")} <span className="font-normal">{selectedRun.startedAt}</span>
                </p>
                <p>
                  {t("Completed:")} <span className="font-normal">{selectedRun.completedAt}</span>
                </p>
              </div>
              <p>
                {t("Ran by:")} <span className="font-normal">{selectedRun.ranBy}</span>
              </p>
              <p>
                {t("Trigger:")} <span className="font-normal">{selectedRun.trigger}</span>
              </p>
              <p>
                {t("Status:")}{" "}
                <span
                  className={`font-normal ${selectedRun.status === "failed" ? "font-bold uppercase text-red-600" : undefined}`}>
                  {selectedRun.status}
                </span>
              </p>
              <Collapsible className="overflow-auto" open={showLogs} onOpenChange={setShowLogs}>
                <CollapsibleTrigger className="flex w-full justify-between">
                  <>
                    <p>{t("Logs:")}</p>
                    {showLogs ? <CaretUp /> : <CaretDown />}
                  </>
                </CollapsibleTrigger>
                <CollapsibleContent>
                  <LogConsole />
                </CollapsibleContent>
              </Collapsible>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export { StatusContent };
