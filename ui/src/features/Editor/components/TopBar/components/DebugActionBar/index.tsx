import { Broom, Play, Stop } from "@phosphor-icons/react";
import { memo } from "react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";

import { DebugStopDialog } from "./components";
import useHooks from "./hooks";

const tooltipOffset = 6;

type Props = {
  onDebugRunStart: () => Promise<void>;
  onDebugRunStop: () => Promise<void>;
};

const DebugActionBar: React.FC<Props> = ({
  onDebugRunStart,
  onDebugRunStop,
}) => {
  const t = useT();
  const {
    showDialog,
    debugRunStarted,
    jobStatus,
    debugJob,
    handleDebugRunStart,
    handleShowDebugStopDialog,
    handleDialogClose,
    handleDebugRunReset,
  } = useHooks({ onDebugRunStart });

  return (
    <>
      <div className="flex rounded-md bg-secondary">
        <div className="flex align-middle gap-2">
          <IconButton
            className="rounded-l-[4px] rounded"
            tooltipText={t("Start debug run of workflow")}
            tooltipOffset={tooltipOffset}
            disabled={
              debugRunStarted ||
              jobStatus === "running" ||
              jobStatus === "queued"
            }
            icon={<Play weight="thin" size={18} />}
            onClick={handleDebugRunStart}
          />
          <IconButton
            tooltipText={t("Stop debug run of workflow")}
            tooltipOffset={tooltipOffset}
            disabled={
              !jobStatus || (jobStatus !== "running" && jobStatus !== "queued")
            }
            icon={<Stop weight="thin" size={18} />}
            onClick={handleShowDebugStopDialog}
          />
          <IconButton
            tooltipText={t("Clear debug run and results")}
            tooltipOffset={tooltipOffset}
            disabled={
              !debugJob ||
              !jobStatus ||
              jobStatus === "running" ||
              jobStatus === "queued"
            }
            icon={<Broom weight="thin" size={18} />}
            onClick={handleDebugRunReset}
          />
        </div>
      </div>
      {showDialog === "debugStop" && (
        <DebugStopDialog
          onDialogClose={handleDialogClose}
          onDebugRunStop={onDebugRunStop}
        />
      )}
    </>
  );
};

export default memo(DebugActionBar);
