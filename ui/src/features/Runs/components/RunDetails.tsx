import { CaretLeft } from "@phosphor-icons/react";
import { useRouter } from "@tanstack/react-router";
import { useCallback } from "react";

import { Button } from "@flow/components";
import { LogConsole } from "@flow/features/Editor/components/BottomPanel/components";
import { useT } from "@flow/lib/i18n";
import { Run } from "@flow/types";

type Props = {
  selectedRun?: Run;
};

const RunDetails: React.FC<Props> = ({ selectedRun }) => {
  const t = useT();
  const router = useRouter();

  const handleBack = useCallback(() => router.history.back(), [router]);

  return (
    selectedRun && (
      <div className="flex flex-1 flex-col gap-4 px-6 pb-2 pt-6">
        <Button size="icon" variant="ghost" onClick={handleBack}>
          <CaretLeft />
        </Button>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
          <div className="rounded-md border font-thin">
            <div className="border-b px-4 py-2">
              <p className="text-xl">{t("Run details")}</p>
            </div>
            <div className="flex flex-col gap-2 p-4">
              <p>
                {t("ID:")} <span className="font-normal">{selectedRun.id}</span>
              </p>
              <p>
                {t("Project Name: ")}
                <span className="font-normal">{selectedRun.project.name}</span>
              </p>
              <div className="flex gap-5">
                <p>
                  {t("Started: ")}
                  <span className="font-normal">{selectedRun.startedAt}</span>
                </p>
                <p>
                  {t("Completed: ")}
                  <span className="font-normal">{selectedRun.completedAt}</span>
                </p>
              </div>
              <p>
                {t("Ran by: ")}
                <span className="font-normal">{selectedRun.ranBy}</span>
              </p>
              <p>
                {t("Trigger: ")}
                <span className="font-normal">{selectedRun.trigger}</span>
              </p>
              <p>
                {t("Status: ")}
                <span
                  className={`font-normal ${selectedRun.status === "failed" ? "font-bold uppercase text-red-600" : undefined}`}
                >
                  {selectedRun.status}
                </span>
              </p>
            </div>
          </div>
          <div className="max-h-[50vh] overflow-auto">
            <LogConsole />
          </div>
        </div>
      </div>
    )
  );
};

export { RunDetails };
