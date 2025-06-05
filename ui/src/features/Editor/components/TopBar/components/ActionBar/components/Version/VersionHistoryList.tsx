import { ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { ProjectDocument, ProjectSnapshotMeta } from "@flow/types";
import { formatDate } from "@flow/utils";

type Props = {
  latestProjectSnapshotVersion?: ProjectDocument;
  history?: ProjectSnapshotMeta[];
  onVersionSelection: (version: number) => void;
  selectedProjectSnapshotVersion: number | null;
};

const VersionHistoryList: React.FC<Props> = ({
  latestProjectSnapshotVersion,
  history,
  selectedProjectSnapshotVersion,
  onVersionSelection,
}) => {
  const t = useT();
  const previousVersions = history?.filter(
    (version) => version.version !== latestProjectSnapshotVersion?.version,
  );

  return (
    <ScrollArea className="h-full w-full overflow-y-auto">
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
            <div>
              <div
                className={`flex cursor-pointer select-none justify-between gap-2 px-2 py-2 ${version.version === selectedProjectSnapshotVersion ? "bg-primary" : "hover:bg-primary"}`}
                onClick={() => onVersionSelection(version.version)}
                style={{ height: "100%" }}>
                <p className="flex-2 self-center text-xs font-thin">
                  {formatDate(version.timestamp)}
                </p>
                <div className="flex justify-end">
                  <p className="rounded border bg-primary/30 p-1 text-xs font-thin">
                    <span className="font-light">
                      {" "}
                      {t("Version ")}
                      {version.version}
                    </span>
                  </p>
                </div>
              </div>
              <div className="h-px bg-primary" />
            </div>
          ))}
        </div>
      ) : null}
      <div className="pt-12" />
    </ScrollArea>
  );
};

export { VersionHistoryList };
