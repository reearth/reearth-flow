import { memo } from "react";

import LogsConsole from "@flow/features/LogsConsole";

type Props = {
  debugJobId?: string;
};
const DebugLogs: React.FC<Props> = ({ debugJobId }) => {
  return debugJobId ? (
    <div className="h-[calc(100%-32px)] overflow-scroll pt-1">
      <LogsConsole jobId={debugJobId} />
    </div>
  ) : null;
};

export default memo(DebugLogs);
