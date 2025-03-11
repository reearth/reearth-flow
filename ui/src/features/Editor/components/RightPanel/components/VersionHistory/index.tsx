import { Doc } from "yjs";

import { FlowLogo, LoadingSkeleton, ScrollArea } from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { useT } from "@flow/lib/i18n";
import { formatDate } from "@flow/utils";

import useHooks from "./hooks";
import { Version } from "./Version";
import { VersionHistoryChangeDialog } from "./VersionHistoryChangeDialog";

type Props = {
  projectId?: string;
  yDoc: Doc | undefined;
};

const VersionHistoryList: React.FC<Props> = ({ projectId, yDoc }) => {
  const t = useT();
  const {
    history,
    isFetching,
    selectedProjectSnapshotVersion,
    latestProjectSnapshotVersion,
    setSelectedProjectSnapshotVersion,
    openVersionChangeDialog,
    setOpenVersionChangeDialog,
    onRollbackProject,
  } = useHooks({ projectId: projectId ?? "", yDoc });

  const previousVersions = history?.filter(
    (version) => version.version !== latestProjectSnapshotVersion?.version,
  );

  const handleVersionClick = (version: number) => {
    setSelectedProjectSnapshotVersion(version);
  };

  const handleDoubleClick = () => {
    setOpenVersionChangeDialog(true);
  };

  return (
    <div className="flex h-full flex-col overflow-auto">
      <ScrollArea>
        {latestProjectSnapshotVersion && (
          <div className="flex items-center justify-between rounded bg-primary p-1 px-4">
            <div>
              <p className="text-sm font-light">{t("Current Version")}</p>
              <p className="flex-[2] text-xs font-thin">
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
          <LoadingSkeleton className="pt-12" />
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
        ) : (
          <BasicBoiler
            text={t("No Versions Available")}
            icon={<FlowLogo className="size-16 text-accent" />}
          />
        )}
      </ScrollArea>
      {openVersionChangeDialog && selectedProjectSnapshotVersion && (
        <VersionHistoryChangeDialog
          selectedProjectSnapshotVersion={selectedProjectSnapshotVersion}
          onDialogClose={() => setOpenVersionChangeDialog(false)}
          onRollbackProject={onRollbackProject}
        />
      )}
    </div>
  );
};

export { VersionHistoryList };
