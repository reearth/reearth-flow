import { ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { ProjectDocument, ProjectSnapshotMeta } from "@flow/types";
import { formatDate } from "@flow/utils";

import { Version } from "./Version";

type Props = {
  latestProjectSnapshotVersion?: ProjectDocument;
  history?: ProjectSnapshotMeta[];
  onVersionSelection: (version: number) => void;
  selectedProjectSnapshotVersion: number | null;
  onPreviewVersion: () => void;
};

const VersionHistoryList: React.FC<Props> = ({
  latestProjectSnapshotVersion,
  history,
  selectedProjectSnapshotVersion,
  onVersionSelection,
  onPreviewVersion,
}) => {
  const t = useT();
  const previousVersions = history?.filter(
    (version) => version.version !== latestProjectSnapshotVersion?.version,
  );

  const handleVersionSelection = (versionNumber: number) => {
    onVersionSelection(versionNumber);
    onPreviewVersion();
  };
  return (
    <ScrollArea className="max-h-[500px] w-full overflow-y-auto place-self-start">
      {latestProjectSnapshotVersion && (
        <div className="flex items-center justify-between bg-primary py-2 px-2">
          <div className="flex flex-col gap-1">
            <p className="text-xs font-light">{t("Current Version")}</p>
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
      {previousVersions && previousVersions.length > 0 ? (
        <div className="flex flex-col overflow-auto">
          {previousVersions?.map((version) => (
            <Version
              key={version.version}
              version={version}
              isSelected={version.version === selectedProjectSnapshotVersion}
              onClick={() => handleVersionSelection(version.version)}
            />
          ))}
        </div>
      ) : null}
      <div className="pt-9" />
    </ScrollArea>
  );
};

export { VersionHistoryList };
