import { MouseEvent, useMemo, useState } from "react";

import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default () => {
  const [expanded, setExpanded] = useState(false);
  const [minimized, setMinimized] = useState(false);

  const [currentProject] = useCurrentProject();

  const { value: debugRunState } = useIndexedDB("debugRun");

  const debugJobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId,
    [debugRunState, currentProject],
  );

  const handleExpand = () => {
    if (minimized) {
      setMinimized(false);
    } else {
      setExpanded((prev) => !prev);
    }
  };

  const handleMinimize = (e: MouseEvent) => {
    e.stopPropagation();
    setMinimized((prev) => !prev);
  };
  return {
    debugJobId,
    expanded,
    minimized,
    handleExpand,
    handleMinimize,
  };
};
