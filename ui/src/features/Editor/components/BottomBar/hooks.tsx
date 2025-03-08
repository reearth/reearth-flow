import { Globe } from "@phosphor-icons/react";
import { useCallback, useMemo, useState } from "react";

import { useShortcuts } from "@flow/hooks";
import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

import { PanelContent } from "./components";
import { ContentID } from "./components/Contents";

type WindowSize = "min" | "max";

export default ({
  isOpen,
  onOpen,
}: {
  isOpen: boolean;
  onOpen: (panel?: "left" | "right" | "bottom") => void;
}) => {
  const t = useT();
  const [windowSize, setWindowSize] = useState<WindowSize>("min");

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

  const panelContentOptions: PanelContent[] = [
    {
      id: "visual-preview",
      button: <Globe className="size-[20px]" weight="thin" />,
      title: t("Preview"),
    },
  ];

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
    selectedId,
    windowSize,
    debugJob,
    panelContentOptions,
    setWindowSize,
    handleSelection,
  };
};
