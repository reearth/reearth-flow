import { memo } from "react";

import DiagnosticsConsole from "@flow/features/DiagnosticsConsole";

type Props = {
  debugJobId?: string;
  isJobActive?: boolean;
};

const DebugDiagnostics: React.FC<Props> = ({ debugJobId, isJobActive }) => {
  return debugJobId ? (
    <div className="h-[calc(100%-32px)] overflow-hidden pt-1">
      <DiagnosticsConsole jobId={debugJobId} isJobActive={isJobActive} />
    </div>
  ) : null;
};

export default memo(DebugDiagnostics);
