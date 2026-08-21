import { XIcon } from "@phosphor-icons/react";
import React, { memo, useCallback, useEffect, useRef, useState } from "react";
import * as Y from "yjs";

import { Button, LoadingSkeleton, LoadingSplashscreen } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Project } from "@flow/types";

import { VersionConfirmationDialog, VersionHistoryList } from "./components";
import VersionEditorComponent from "./components/VersionEditorComponent";
import useHooks from "./hooks";

type Props = {
  project?: Project;
  yDoc: Y.Doc | null;
  onDialogClose: () => void;
};

// Normal mode: user-meaningful named versions, with preview on select and
// restore. Both are addressed by snapshotNumber through projectNamedSnapshot;
// see ./hooks.ts for why that must never be confused with the update-log clock.
//
// The project-corruption recovery flow is a separate component working off the
// raw update log: ./RecoveryDialog.tsx.
const VersionDialog: React.FC<Props> = ({ project, yDoc, onDialogClose }) => {
  const t = useT();
  const dialogRef = useRef<HTMLDivElement>(null);
  const [animate, setAnimate] = useState<boolean>(false);
  const {
    snapshots,
    latestProjectSnapshotVersion,
    isFetching,
    isError,
    previewDocYWorkflows,
    selectedSnapshotNumber,
    isLoadingPreview,
    isRestoring,
    openVersionConfirmationDialog,
    setOpenVersionConfirmationDialog,
    onSnapshotSelect,
    onSnapshotRestore,
    destroyPreview,
  } = useHooks({
    projectId: project?.id ?? "",
    yDoc,
    onDialogClose,
  });

  const handleDialogClose = useCallback(() => {
    destroyPreview();
    setAnimate(false);
    onDialogClose();
  }, [destroyPreview, onDialogClose]);

  useEffect(() => {
    setAnimate(true);
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as Element;

      const isDropdownClick = target?.closest?.(
        '[data-slot="dropdown-menu-content"]',
      );

      const isDialogClick = target?.closest?.('[data-slot="dialog-content"]');

      if (
        dialogRef.current &&
        !isDialogClick &&
        !dialogRef.current.contains(event.target as Node) &&
        !isDropdownClick &&
        !openVersionConfirmationDialog
      ) {
        handleDialogClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [handleDialogClose, openVersionConfirmationDialog]);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      role="dialog"
      aria-modal="true">
      <div
        ref={dialogRef}
        className={`relative flex h-[90vh] w-[90vw] flex-col overflow-hidden rounded-lg bg-card shadow-lg transition-all duration-170 ease-in-out  ${animate ? "scale-100 opacity-100" : "scale-95 opacity-0"}`}>
        <div className="flex items-center justify-between p-6">
          <h2 className="rounded-t-lg text-xl leading-none tracking-tight dark:font-thin">
            {selectedSnapshotNumber !== null
              ? t("Viewing Snapshot: {{snapshotNumber}}", {
                  snapshotNumber: selectedSnapshotNumber,
                })
              : t("Viewing Version: {{version}}", {
                  version: latestProjectSnapshotVersion?.version,
                })}
          </h2>
          <Button
            variant={"ghost"}
            className="z-10 h-fit p-0 opacity-70 hover:bg-card hover:opacity-100 dark:font-thin"
            onClick={handleDialogClose}>
            <XIcon className="size-5" />
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
              />
            )}
          </div>
          <div className="relative flex h-full w-[30vw] max-w-[500px] min-w-[320px] flex-col">
            <div className="text-md pt-4 pl-4 dark:font-thin">
              {t("Version History")}
            </div>
            <div className="flex-1 overflow-y-auto p-4 pb-[55px]">
              {isFetching ? (
                <LoadingSkeleton />
              ) : (
                <VersionHistoryList
                  latestProjectSnapshotVersion={latestProjectSnapshotVersion}
                  snapshots={snapshots}
                  isError={isError}
                  selectedSnapshotNumber={selectedSnapshotNumber}
                  onSnapshotSelect={onSnapshotSelect}
                />
              )}
            </div>
            <div className="absolute bottom-0 left-0 flex w-full justify-end border-t border-accent bg-secondary p-2">
              <Button
                disabled={selectedSnapshotNumber === null || isLoadingPreview}
                variant={"ghost"}
                onClick={() => setOpenVersionConfirmationDialog(true)}>
                {t("Restore")}
              </Button>
            </div>
          </div>
        </div>
      </div>

      {isRestoring && <LoadingSplashscreen />}
      {openVersionConfirmationDialog &&
        selectedSnapshotNumber !== null &&
        !isRestoring && (
          <VersionConfirmationDialog
            selectedProjectSnapshotVersion={selectedSnapshotNumber}
            onDialogClose={() => setOpenVersionConfirmationDialog(false)}
            onProjectRollback={onSnapshotRestore}
          />
        )}
    </div>
  );
};

export default memo(VersionDialog);
