import { Cross2Icon } from "@radix-ui/react-icons";
import { ReactFlowProvider } from "@xyflow/react";
import React, { useCallback, useEffect, useRef, useState } from "react";
import { ErrorBoundary } from "react-error-boundary";
import * as Y from "yjs";

import {
  Button,
  LoadingSplashscreen,
  LoadingSkeleton,
  FlowLogo,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import VersionCanvas from "@flow/features/VersionCanvas";
import { useT } from "@flow/lib/i18n";
import type { YWorkflow } from "@flow/lib/yjs/types";
import type { Project } from "@flow/types";

import useHooks from "./hooks";
import { VersionConfirmationDialog } from "./VersionConfirmationDialog";
import { VersionHistoryList } from "./VersionHistoryList";

type Props = {
  project?: Project;
  yDoc: Y.Doc | null;
  onDialogClose: () => void;
};

const VersionDialog: React.FC<Props> = ({ project, yDoc, onDialogClose }) => {
  const t = useT();
  const dialogRef = useRef<HTMLDivElement>(null);
  const [animate, setAnimate] = useState<boolean>(false);
  const lastErroredVersionRef = useRef<number | null>(null);
  const [isCorruptedVersion, setIsCorruptedVersion] = useState<boolean>(false);
  const [isCorruptionDetected, setIsCorruptionDetected] =
    useState<boolean>(false);
  const {
    history,
    latestProjectSnapshotVersion,
    previewDocRef,
    previewDocYWorkflows,
    selectedProjectSnapshotVersion,
    isFetching,
    isLoadingPreview,
    isReverting,
    openVersionConfirmationDialog,
    setOpenVersionConfirmationDialog,
    onRollbackProject,
    onVersionSelection,
  } = useHooks({ projectId: project?.id ?? "", yDoc, onDialogClose });

  const handleCloseDialog = useCallback(() => {
    previewDocRef.current?.destroy();
    previewDocRef.current = null;
    setAnimate(false);
    onDialogClose();
  }, [previewDocRef, onDialogClose]);

  const handleWorkflowCorruption = useCallback(() => {
    lastErroredVersionRef.current = selectedProjectSnapshotVersion;
    setIsCorruptedVersion(true);
    setIsCorruptionDetected(true);
  }, [selectedProjectSnapshotVersion]);

  const handleRollbackProject = useCallback(async () => {
    try {
      await onRollbackProject();

      if (isCorruptionDetected) {
        window.location.reload();
      }
    } catch (error) {
      console.error("Rollback failed:", error);
    }
  }, [onRollbackProject, isCorruptionDetected]);

  useEffect(() => {
    setAnimate(true);
    if (
      isCorruptedVersion &&
      selectedProjectSnapshotVersion !== lastErroredVersionRef.current
    ) {
      setIsCorruptedVersion(false);
    }
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dialogRef.current &&
        !dialogRef.current.contains(event.target as Node) &&
        !openVersionConfirmationDialog
      ) {
        handleCloseDialog();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [
    handleCloseDialog,
    openVersionConfirmationDialog,
    selectedProjectSnapshotVersion,
    isCorruptedVersion,
  ]);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      role="dialog"
      aria-modal="true">
      <div
        ref={dialogRef}
        className={`w-[90vw] h-[90vh] bg-card shadow-lg rounded-lg flex flex-col overflow-hidden relative transition-all duration-170 ease-in-out  ${animate ? "opacity-100 scale-100" : "opacity-0 scale-95"}`}>
        <div className="flex p-6 items-center justify-between border-b">
          <h2 className="text-xl dark:font-thin leading-none tracking-tight rounded-t-lg">
            {t("Viewing Version: {{version}}", {
              version:
                selectedProjectSnapshotVersion ??
                latestProjectSnapshotVersion?.version,
            })}
          </h2>
          <Button
            variant={"ghost"}
            className="h-fit p-0 opacity-70 dark:font-thin hover:bg-card hover:opacity-100 z-10"
            onClick={handleCloseDialog}>
            <Cross2Icon className="size-5" />
          </Button>
        </div>
        <div className="flex flex-1 overflow-hidden">
          <div className="flex-1 overflow-auto">
            {isLoadingPreview ? (
              <LoadingSkeleton className="w-full h-full" />
            ) : (
              <VersionEditorComponent
                yDoc={yDoc}
                previewDocYWorkflows={previewDocYWorkflows}
                onWorkflowCorruption={handleWorkflowCorruption}
              />
            )}
          </div>
          <div className="w-[30vw] min-w-[320px] max-w-[500px] h-full border-l flex flex-col relative">
            <div className="text-md dark:font-thin pl-4 pt-4">
              {t("Version History")}
            </div>
            <div className="flex-1 overflow-y-auto p-4 pb-[55px]">
              {isFetching ? (
                <LoadingSkeleton />
              ) : (
                <VersionHistoryList
                  latestProjectSnapshotVersion={latestProjectSnapshotVersion}
                  history={history}
                  selectedProjectSnapshotVersion={
                    selectedProjectSnapshotVersion
                  }
                  onVersionSelection={onVersionSelection}
                />
              )}
            </div>
            <div className="absolute bottom-0 left-0 w-full bg-secondary border-t p-2 flex justify-end">
              <Button
                disabled={!selectedProjectSnapshotVersion || isCorruptedVersion}
                variant={"ghost"}
                onClick={() => setOpenVersionConfirmationDialog(true)}>
                {t("Revert")}
              </Button>
            </div>
          </div>
        </div>
      </div>

      {isReverting && <LoadingSplashscreen />}
      {openVersionConfirmationDialog &&
        selectedProjectSnapshotVersion &&
        !isReverting && (
          <VersionConfirmationDialog
            selectedProjectSnapshotVersion={selectedProjectSnapshotVersion}
            onDialogClose={() => setOpenVersionConfirmationDialog(false)}
            onRollbackProject={handleRollbackProject}
          />
        )}
    </div>
  );
};

const VersionEditorComponent: React.FC<{
  yDoc: Y.Doc | null;
  previewDocYWorkflows: Y.Map<YWorkflow> | null;
  onWorkflowCorruption?: () => void;
}> = ({ yDoc, previewDocYWorkflows, onWorkflowCorruption }) => {
  const t = useT();
  const yWorkflows = previewDocYWorkflows
    ? previewDocYWorkflows
    : yDoc
      ? yDoc.getMap<YWorkflow>("workflows")
      : null;

  return (
    <div className="w-full h-full">
      {yWorkflows && (
        <ErrorBoundary
          onError={onWorkflowCorruption}
          fallback={
            <BasicBoiler
              text={t("Selected version is corrupted or not available.")}
              className="size-4 h-full [&>div>p]:text-md"
              icon={<FlowLogo className="size-20 text-accent" />}
            />
          }>
          <ReactFlowProvider>
            <VersionCanvas yWorkflows={yWorkflows} />
          </ReactFlowProvider>
        </ErrorBoundary>
      )}
    </div>
  );
};

export { VersionDialog };
