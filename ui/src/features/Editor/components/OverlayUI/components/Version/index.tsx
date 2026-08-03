import { XIcon } from "@phosphor-icons/react";
import React, { memo, useCallback, useEffect, useRef, useState } from "react";
import * as Y from "yjs";

import { Button, LoadingSkeleton } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Project } from "@flow/types";

import { VersionHistoryList } from "./components";
import VersionEditorComponent from "./components/VersionEditorComponent";
import useHooks from "./hooks";

type Props = {
  project?: Project;
  yDoc: Y.Doc | null;
  onDialogClose: () => void;
  // Accepted for backward compatibility with the project-corruption error
  // boundary (see workspaces.$workspaceId_.projects_.$projectId.lazy.tsx),
  // which renders this dialog with a "Revert to a previous version" call
  // to action. It is intentionally unused here: Revert is disabled for
  // snapshot-backed rows (see ./hooks.ts), so there is currently no
  // in-dialog action that would need to reset that boundary. Recovering a
  // corrupted project via this dialog will not work again until snapshot
  // preview/restore is wired up.
  onErrorReset?: () => void;
};

const VersionDialog: React.FC<Props> = ({ project, yDoc, onDialogClose }) => {
  const t = useT();
  const dialogRef = useRef<HTMLDivElement>(null);
  const [animate, setAnimate] = useState<boolean>(false);
  const { snapshots, latestProjectSnapshotVersion, isFetching } = useHooks({
    projectId: project?.id ?? "",
  });

  const handleDialogClose = useCallback(() => {
    setAnimate(false);
    onDialogClose();
  }, [onDialogClose]);

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
        !isDropdownClick
      ) {
        handleDialogClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [handleDialogClose]);

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
            {t("Viewing Version: {{version}}", {
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
            {/* Snapshot rows have no preview capability yet (see
            ./hooks.ts), so this always shows the live document. */}
            <VersionEditorComponent yDoc={yDoc} previewDocYWorkflows={null} />
          </div>
          <div className="relative flex h-full w-[30vw] max-w-[500px] min-w-[320px] flex-col">
            <div className="text-md pt-4 pl-4 dark:font-thin">
              {t("Version History")}
            </div>
            <div className="flex-1 overflow-y-auto p-4">
              {isFetching ? (
                <LoadingSkeleton />
              ) : (
                <VersionHistoryList
                  latestProjectSnapshotVersion={latestProjectSnapshotVersion}
                  snapshots={snapshots}
                />
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default memo(VersionDialog);
