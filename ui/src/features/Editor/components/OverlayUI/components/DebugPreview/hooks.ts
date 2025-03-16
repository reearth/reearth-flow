import { MouseEvent, useEffect, useMemo, useState } from "react";

import useFetchAndReadData from "@flow/hooks/useFetchAndReadData";
import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default () => {
  const t = useT();
  const [expanded, setExpanded] = useState(false);
  const [minimized, setMinimized] = useState(false);

  const [currentProject] = useCurrentProject();

  const { value: debugRunState } = useIndexedDB("debugRun");

  const debugJob = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );

  const { useGetJob } = useJob();

  const outputURLs = useGetJob(debugJob?.jobId ?? "").job?.outputURLs;

  const intermediateDataURL = useMemo(
    () =>
      debugJob?.selectedIntermediateData?.url ??
      "/7571eea0-eabf-4ff7-b978-e5965d882409.jsonl", // TODO: Remove this default value
    [debugJob],
  );

  const dataURLs = useMemo(() => {
    const urls: { key: string; name: string }[] = [];
    if (intermediateDataURL) {
      urls.push({
        key: intermediateDataURL,
        name: intermediateDataURL.split("/").pop() || intermediateDataURL,
      });
    }
    if (outputURLs) {
      urls.push(
        ...outputURLs.map((url) => ({
          key: url,
          name: url.split("/").pop() + `(${t("Output data")})`,
        })),
      );
    }
    return urls.length ? urls : undefined;
  }, [outputURLs, intermediateDataURL, t]);

  const [selectedDataURL, setSelectedDataURL] = useState<string | null>(null);

  useEffect(() => {
    if (dataURLs?.length && !selectedDataURL) {
      setSelectedDataURL(dataURLs[0].key);
    }
  }, [dataURLs, selectedDataURL]);

  const handleSelectedDataChange = (url: string) => {
    setSelectedDataURL(url);
  };

  const { fileContent: selectedOutputData, fileType } = useFetchAndReadData({
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
    dataURLs,
    expanded,
    minimized,
    selectedOutputData,
    fileType,
    handleExpand,
    handleMinimize,
    handleTabChange,
    handleSelectedDataChange,
  };
};
