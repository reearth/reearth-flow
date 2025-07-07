import { memo } from "react";

import LogsConsole from "@flow/features/LogsConsole";

import useHooks from "./hooks";

const DebugLogs: React.FC = () => {
  const { debugJobId } = useHooks();

  return debugJobId ? (
    <div className="h-[calc(100%-32px)] overflow-scroll pt-1">
      <LogsConsole jobId={debugJobId} />
    </div>
  ) : null;
};

export default memo(DebugLogs);
