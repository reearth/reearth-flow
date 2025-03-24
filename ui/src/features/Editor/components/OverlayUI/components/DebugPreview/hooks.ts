import { MouseEvent, useEffect, useMemo, useRef, useState } from "react";

import useFetchAndReadData from "@flow/hooks/useFetchAndReadData";
import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default () => {
  const t = useT();

  const prevIntermediateDataUrl = useRef<string | undefined>(undefined);
  const [expanded, setExpanded] = useState(false);
  const [minimized, setMinimized] = useState(false);

  const [currentProject] = useCurrentProject();

  const { value: debugRunState } = useIndexedDB("debugRun");

  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );

  const { useGetJob } = useJob();

  const { job: debugJob, refetch } = useGetJob(debugJobState?.jobId ?? "");

  const outputURLs = useMemo(() => debugJob?.outputURLs, [debugJob]);

  useEffect(() => {
    if (
      !outputURLs &&
      (debugJobState?.status === "completed" ||
        debugJobState?.status === "failed" ||
        debugJobState?.status === "cancelled")
    ) {
      (async () => {
        try {
          await refetch();
        } catch (error) {
          console.error("Error during refetch:", error);
        }
      })();
    }
  }, [debugJobState?.status, outputURLs, refetch]);

  const intermediateDataURL = useMemo(
    () => debugJobState?.selectedIntermediateData?.url,
    [debugJobState],
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

  const [selectedDataURL, setSelectedDataURL] = useState<string | undefined>(
    undefined,
  );

  useEffect(() => {
    if (intermediateDataURL !== prevIntermediateDataUrl.current) {
      setSelectedDataURL(intermediateDataURL);
      prevIntermediateDataUrl.current = intermediateDataURL;
      setMinimized(false);
    } else if (
      (dataURLs?.length && !selectedDataURL) ||
      (selectedDataURL && !dataURLs?.find((u) => u.key === selectedDataURL))
    ) {
      setSelectedDataURL(dataURLs?.[0].key);
    }
  }, [dataURLs, selectedDataURL, intermediateDataURL]);

  const handleSelectedDataChange = (url: string) => {
    setSelectedDataURL(url);
    setMinimized(false);
  };

  const {
    fileContent: selectedOutputData,
    fileType,
    isLoading: isLoadingData,
  } = useFetchAndReadData({
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
    selectedDataURL,
    dataURLs,
    expanded,
    minimized,
    selectedOutputData,
    fileType,
    debugJobState,
    isLoadingData,
    handleExpand,
    handleMinimize,
    handleTabChange,
    handleSelectedDataChange,
  };
};
