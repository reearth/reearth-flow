import { Cross2Icon } from "@radix-ui/react-icons";
import React, { memo, useCallback, useEffect, useRef, useState } from "react";
import * as Y from "yjs";

import { Button, LoadingSplashscreen, LoadingSkeleton } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Project } from "@flow/types";

import { VersionConfirmationDialog, VersionHistoryList } from "./components";
import VersionEditorComponent from "./components/VersionEditorComponent";
import useHooks from "./hooks";

type Props = {
  project?: Project;
  yDoc: Y.Doc | null;
  onDialogClose: () => void;
  onErrorReset?: () => void;
};

const VersionDialog: React.FC<Props> = ({
  project,
  yDoc,
  onDialogClose,
  onErrorReset,
}) => {
  const t = useT();
  const dialogRef = useRef<HTMLDivElement>(null);
  const [animate, setAnimate] = useState<boolean>(false);

  const {
    history,
    latestProjectSnapshotVersion,
    previewDocRef,
    previewDocYWorkflows,
    selectedProjectSnapshotVersion,
    isFetching,
    isLoadingPreview,
    isReverting,
    isCorruptedVersion,
    openVersionConfirmationDialog,
    setOpenVersionConfirmationDialog,
    onProjectRollback,
    onVersionSelection,
    onWorkflowCorruption,
  } = useHooks({ projectId: project?.id ?? "", yDoc, onDialogClose });

  const handleDialogClose = useCallback(() => {
    previewDocRef.current?.destroy();
    previewDocRef.current = null;
    setAnimate(false);
    onDialogClose();
  }, [previewDocRef, onDialogClose]);

  const handleProjectRollback = useCallback(async () => {
    try {
      await onProjectRollback();
      if (onErrorReset) {
        onErrorReset();
      }
    } catch (error) {
      console.error("Rollback failed:", error);
    }
  }, [onProjectRollback, onErrorReset]);

  useEffect(() => {
    setAnimate(true);
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dialogRef.current &&
        !dialogRef.current.contains(event.target as Node) &&
        !openVersionConfirmationDialog
      ) {
        handleDialogClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [
    handleDialogClose,
    openVersionConfirmationDialog,
    selectedProjectSnapshotVersion,
  ]);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      role="dialog"
      aria-modal="true">
      <div
        ref={dialogRef}
        className={`relative flex h-[90vh] w-[90vw] flex-col overflow-hidden rounded-lg bg-card shadow-lg transition-all duration-170 ease-in-out  ${animate ? "scale-100 opacity-100" : "scale-95 opacity-0"}`}>
        <div className="flex items-center justify-between border-b p-6">
          <h2 className="rounded-t-lg text-xl leading-none tracking-tight dark:font-thin">
            {t("Viewing Version: {{version}}", {
              version:
                selectedProjectSnapshotVersion ??
                latestProjectSnapshotVersion?.version,
            })}
          </h2>
          <Button
            variant={"ghost"}
            className="z-10 h-fit p-0 opacity-70 hover:bg-card hover:opacity-100 dark:font-thin"
            onClick={handleDialogClose}>
            <Cross2Icon className="size-5" />
          </Button>
        </div>
        <div className="flex flex-1 overflow-hidden">
          <div className="flex-1 overflow-auto">
            {isLoadingPreview ? (
              <LoadingSkeleton className="h-full w-full" />
            ) : (
              <VersionEditorComponent
                yDoc={yDoc}
                previewDocYWorkflows={previewDocYWorkflows}
                onWorkflowCorruption={onWorkflowCorruption}
              />
            )}
          </div>
          <div className="relative flex h-full w-[30vw] max-w-[500px] min-w-[320px] flex-col border-l">
            <div className="text-md pt-4 pl-4 dark:font-thin">
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
            <div className="absolute bottom-0 left-0 flex w-full justify-end border-t bg-secondary p-2">
              <Button
                disabled={
                  !selectedProjectSnapshotVersion ||
                  isLoadingPreview ||
                  isCorruptedVersion
                }
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
            onProjectRollback={handleProjectRollback}
          />
        )}
    </div>
  );
};

export default memo(VersionDialog);
