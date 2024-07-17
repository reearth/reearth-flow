import { CaretDown, CaretUp } from "@phosphor-icons/react";
import { useState } from "react";

import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Run } from "@flow/types";

import { LogConsole } from "../../BottomPanel/components";

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
      <div className="flex gap-2 items-center text-lg font-extralight">
        <p>{t("Runs Manager")}</p>
        <p className="text-sm font-thin text-zinc-400">({label})</p>
      </div>
      <div className="flex flex-col gap-6 mt-4 max-w-[1200px]">
        <div className="max-h-[30vh] overflow-auto rounded-md px-2">
          <RunsTable runs={runs} selectedRun={selectedRun} onRunSelect={onRunSelect} />
        </div>
        {selectedRun && (
          <div className="rounded-md border border-zinc-700 mx-4 text-zinc-300 font-thin">
            <div className="py-2 px-4 border-b border-zinc-700">
              <p className="text-xl">{t("Run details")}</p>
            </div>
            <div className="max-h-[45vh] flex flex-col gap-2 p-4">
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
                  className={`font-normal ${selectedRun.status === "failed" ? "text-red-600 font-bold uppercase" : undefined}`}>
                  {selectedRun.status}
                </span>
              </p>
              <Collapsible className="overflow-auto" open={showLogs} onOpenChange={setShowLogs}>
                <CollapsibleTrigger className="flex justify-between w-full">
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
