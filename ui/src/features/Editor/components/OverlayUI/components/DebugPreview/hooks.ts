import { MouseEvent, useEffect, useMemo, useState } from "react";

import useFetchAndReadData from "@flow/hooks/useFetchAndReadData";
import { useJob } from "@flow/lib/gql/job";
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

  const { useGetJob } = useJob();

  const outputURLs = useGetJob(debugJobId ?? "").job?.outputURLs;

  const [selectedDataURL, setSelectedDataURL] = useState<string | null>(null);

  useEffect(() => {
    if (outputURLs?.length && !selectedDataURL) {
      setSelectedDataURL(outputURLs[0]);
    }
  }, [outputURLs, selectedDataURL]);

  const handleSelectedDataChange = (url: string) => {
    console.log("url", url);
    setSelectedDataURL(url);
  };

  const { fileContent, fileType } = useFetchAndReadData({
    dataUrl: selectedDataURL ?? "",
  });

  const handleExpand = () => {
    setExpanded((prev) => !prev);
  };

  const handleMinimize = (e: MouseEvent) => {
    e.stopPropagation();
    setMinimized((prev) => !prev);
  };

  const handleTabChange = () => {
    if (minimized) {
      setMinimized(false);
    }
  };

  return {
    outputURLs,
    expanded,
    minimized,
    fileContent,
    fileType,
    handleExpand,
    handleMinimize,
    handleTabChange,
    handleSelectedDataChange,
  };
};
