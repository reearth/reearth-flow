import {
  CaretUpIcon,
  CaretDownIcon,
  MinusIcon,
  TerminalIcon,
} from "@phosphor-icons/react";
import { memo } from "react";

import LogsConsole from "@flow/features/LogsConsole";
import { useT } from "@flow/lib/i18n";

import useHooks from "./hooks";

const DebugLogs: React.FC = () => {
  const t = useT();
  const { debugJobId, expanded, minimized, handleExpand, handleMinimize } =
    useHooks();

  return debugJobId ? (
    <div
      className={`pointer-events-auto w-[45vw] min-w-[700px] cursor-pointer rounded-md bg-secondary shadow-md shadow-secondary transition-all ${minimized ? "h-[36px]" : expanded ? "h-[90vh]" : "h-[350px]"}`}>
      <div className="flex items-center p-1" onClick={handleExpand}>
        <div className="flex flex-1 items-center justify-center gap-2">
          <TerminalIcon />
          <p className="text-sm font-thin select-none">{t("Workflow Logs")}</p>
        </div>
        <div className="flex items-center gap-2">
          <div
            className="rounded p-1 hover:bg-primary"
            onClick={handleMinimize}>
            {minimized ? (
              <CaretUpIcon weight="light" />
            ) : (
              <MinusIcon weight="light" />
            )}
          </div>
          {!minimized && (
            <div className="rounded p-1 hover:bg-primary">
              {expanded ? (
                <CaretDownIcon weight="light" />
              ) : (
                <CaretUpIcon weight="light" />
              )}
            </div>
          )}
        </div>
      </div>
      <div className="h-[calc(100%-32px)] overflow-scroll pt-1">
        <LogsConsole jobId={debugJobId} />
      </div>
    </div>
  ) : null;
};

export default memo(DebugLogs);
