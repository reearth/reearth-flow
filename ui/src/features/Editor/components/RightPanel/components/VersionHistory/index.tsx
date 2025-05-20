import { Doc } from "yjs";

import {
  LoadingSkeleton,
  LoadingSplashscreen,
  ScrollArea,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Project } from "@flow/types";
import { formatDate } from "@flow/utils";

import useHooks from "./hooks";
import { Version } from "./Version";
import { VersionConfirmationDialog } from "./VersionConfirmationDialog";
import { VersionHistoryChangeDialog } from "./VersionHistoryChangeDialog";

type Props = {
  project?: Project;
  yDoc: Doc | null;
};

const VersionHistoryList: React.FC<Props> = ({ project, yDoc }) => {
  const t = useT();
  const {
    history,
    isFetching,
    isReverting,
    selectedProjectSnapshotVersion,
    latestProjectSnapshotVersion,
    setSelectedProjectSnapshotVersion,
    versionPreviewYWorkflows,
    openVersionChangeDialog,
    openVersionPreviewDialog,
    setOpenVersionChangeDialog,
    setOpenVersionPreviewDialog,
    onRollbackProject,
    onPreviewVersion,
  } = useHooks({ projectId: project?.id ?? "", yDoc });

  const previousVersions = history?.filter(
    (version) => version.version !== latestProjectSnapshotVersion?.version,
  );

  const handleVersionClick = (version: number) => {
    setSelectedProjectSnapshotVersion(version);
  };

  const handleDoubleClick = () => {
    setOpenVersionPreviewDialog(true);
    onPreviewVersion();
  };

  return (
    <div className="flex h-full flex-col overflow-auto">
      <ScrollArea>
        {latestProjectSnapshotVersion && (
          <div className="flex items-center justify-between rounded bg-primary p-1 px-4">
            <div>
              <p className="text-sm font-light">{t("Current Version")}</p>
              <p className="flex-2 text-xs font-thin">
                {formatDate(latestProjectSnapshotVersion.timestamp)}
              </p>
            </div>
            <p className="rounded border bg-logo/30 p-1 text-xs font-thin">
              <span className="font-light">
                {" "}
                {t("Version ")}
                {latestProjectSnapshotVersion.version}
              </span>
            </p>
          </div>
        )}
        {isFetching ? (
          <LoadingSkeleton className="h-[75vh] pt-12" />
        ) : previousVersions && previousVersions.length > 0 ? (
          <div className="flex flex-col overflow-auto">
            {previousVersions?.map((version) => (
              <Version
                // key={version}
                version={version}
                isSelected={version.version === selectedProjectSnapshotVersion}
                onClick={() => handleVersionClick(version.version)}
                onDoubleClick={handleDoubleClick}
              />
            ))}
            <div className="pb-8" />
          </div>
        ) : null}
      </ScrollArea>
      {openVersionPreviewDialog && selectedProjectSnapshotVersion && (
        <VersionHistoryChangeDialog
          selectedProjectSnapshotVersion={selectedProjectSnapshotVersion}
          versionPreviewYWorkflows={versionPreviewYWorkflows}
          onDialogClose={() => setOpenVersionPreviewDialog(false)}
          onVersionConfirmationDialogOpen={() =>
            setOpenVersionChangeDialog(true)
          }
        />
      )}
      {openVersionChangeDialog &&
        selectedProjectSnapshotVersion &&
        !isReverting && (
          <VersionConfirmationDialog
            selectedProjectSnapshotVersion={selectedProjectSnapshotVersion}
            onDialogClose={() => setOpenVersionChangeDialog(false)}
            onRollbackProject={onRollbackProject}
          />
        )}
      {isReverting && <LoadingSplashscreen />}
    </div>
  );
};

export { VersionHistoryList };
