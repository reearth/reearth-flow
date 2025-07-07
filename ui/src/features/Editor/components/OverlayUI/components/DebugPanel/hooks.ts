import {
  MouseEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import useFetchAndReadData from "@flow/hooks/useFetchAndReadData";
import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default () => {
  const t = useT();

  const prevIntermediateDataUrls = useRef<string[] | undefined>(undefined);
  const [expanded, setExpanded] = useState(false);
  const [minimized, setMinimized] = useState(false);

  const [currentProject] = useCurrentProject();

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );
  const debugJobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId,
    [debugRunState, currentProject],
  );

  const [showTempPossibleIssuesDialog, setShowTempPossibleIssuesDialog] =
    useState(false);

  const { useGetJob } = useJob();

  const { job: debugJob, refetch } = useGetJob(debugJobState?.jobId ?? "");

  const outputURLs = useMemo(() => debugJob?.outputURLs, [debugJob]);

  const handleShowTempPossibleIssuesDialogClose = useCallback(() => {
    updateValue((prevState) => {
      const newJobs = prevState.jobs.map((pj) => {
        if (
          debugJob?.id === pj.jobId &&
          !pj.tempWorkflowHasPossibleIssuesFlag
        ) {
          return {
            ...pj,
            tempWorkflowHasPossibleIssuesFlag: false,
          };
        } else {
          return pj;
        }
      });
      return {
        jobs: newJobs,
      };
    });
    setShowTempPossibleIssuesDialog(false);
  }, [debugJob?.id, updateValue]);

  useEffect(() => {
    if (debugJobState?.tempWorkflowHasPossibleIssuesFlag) return;
    if (
      !outputURLs &&
      (debugJobState?.status === "completed" ||
        debugJobState?.status === "failed" ||
        debugJobState?.status === "cancelled")
    ) {
      (async () => {
        try {
          const { data: job } = await refetch();

          if (
            !job?.outputURLs &&
            debugJobState?.tempWorkflowHasPossibleIssuesFlag === undefined
          ) {
            updateValue((prevState) => {
              const newJobs = prevState.jobs.map((pj) => {
                if (
                  job?.id === pj.jobId &&
                  !pj.tempWorkflowHasPossibleIssuesFlag
                ) {
                  const tempFlag = !job.outputURLs?.length;
                  setShowTempPossibleIssuesDialog(tempFlag);
                  return {
                    ...pj,
                    tempWorkflowHasPossibleIssuesFlag: tempFlag, // No logsURL + a completed/failed/cancelled status means potential issues. @KaWaite
                  };
                } else {
                  return pj;
                }
              });
              return {
                jobs: newJobs,
              };
            });
          }
        } catch (error) {
          console.error("Error during refetch:", error);
        }
      })();
    }
  }, [
    debugJobState?.status,
    debugJobState?.tempWorkflowHasPossibleIssuesFlag,
    outputURLs,
    refetch,
    updateValue,
  ]);

  const intermediateDataURLs = useMemo(
    () => debugJobState?.selectedIntermediateData?.map((sid) => sid.url),
    [debugJobState],
  );

  const dataURLs = useMemo(() => {
    const urls: { key: string; name: string }[] = [];
    if (intermediateDataURLs) {
      intermediateDataURLs.forEach((intermediateDataURL) => {
        urls.push({
          key: intermediateDataURL,
          name: intermediateDataURL.split("/").pop() || intermediateDataURL,
        });
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
  }, [outputURLs, intermediateDataURLs, t]);

  const [selectedDataURL, setSelectedDataURL] = useState<string | undefined>(
    undefined,
  );

  useEffect(() => {
    if (intermediateDataURLs !== prevIntermediateDataUrls.current) {
      const newURL = intermediateDataURLs?.find(
        (url) => !prevIntermediateDataUrls.current?.includes(url),
      );
      setSelectedDataURL(newURL);
      prevIntermediateDataUrls.current = intermediateDataURLs;
      setMinimized(false);
    } else if (
      (dataURLs?.length && !selectedDataURL) ||
      (selectedDataURL && !dataURLs?.find((u) => u.key === selectedDataURL))
    ) {
      setSelectedDataURL(dataURLs?.[0].key);
    }
  }, [dataURLs, selectedDataURL, intermediateDataURLs]);

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
    debugJobId,
    selectedDataURL,
    dataURLs,
    expanded,
    minimized,
    selectedOutputData,
    fileType,
    debugJobState,
    isLoadingData,
    showTempPossibleIssuesDialog,
    handleShowTempPossibleIssuesDialogClose,
    handleExpand,
    handleMinimize,
    handleTabChange,
    handleSelectedDataChange,
  };
};
