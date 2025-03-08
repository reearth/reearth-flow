import { useCallback, useMemo, useState } from "react";

import { useShortcuts } from "@flow/hooks";
import { useJob } from "@flow/lib/gql/job";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

import { ContentID } from "./components/Contents";

export default ({
  isOpen,
  onOpen,
}: {
  isOpen: boolean;
  onOpen: (panel?: "left" | "right" | "bottom") => void;
}) => {
  const [currentProject] = useCurrentProject();
  const { useGetJob } = useJob();

  const { value: debugRunState } = useIndexedDB("debugRun");

  const debugJobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId ?? "",
    [debugRunState, currentProject],
  );

  const debugJob = useGetJob(debugJobId).job;

  const handlePanelToggle = useCallback(
    (open: boolean) => onOpen(open ? "bottom" : undefined),
    [onOpen],
  );

  const [selectedId, setSelectedId] = useState<ContentID | undefined>(
    undefined,
  );

  const handleSelection = useCallback(
    (id: ContentID) => {
      if (id !== selectedId) {
        setSelectedId(id);
        if (!isOpen) {
          handlePanelToggle?.(true);
        }
      } else {
        handlePanelToggle?.(!isOpen);
      }
    },
    [isOpen, handlePanelToggle, selectedId, setSelectedId],
  );

  useShortcuts([
    {
      keyBinding: { key: "p", commandKey: true },
      callback: () => {
        handleSelection("visual-preview");
      },
    },
  ]);

  return {
    debugJob,
  };
};
