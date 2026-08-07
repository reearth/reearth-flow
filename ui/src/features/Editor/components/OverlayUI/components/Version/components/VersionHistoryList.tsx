import { ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { AUTO_SNAPSHOT_LABEL } from "@flow/types";
import type { NamedSnapshot, ProjectDocument } from "@flow/types";
import { formatDate } from "@flow/utils";

type Props = {
  latestProjectSnapshotVersion?: ProjectDocument;
  snapshots?: NamedSnapshot[];
  isError?: boolean;
};

// Read-only: NamedSnapshot.id is a different, backend-assigned ID space
// than the raw update-log `version` that preview/revert are built on (see
// the comment in ../hooks.ts). There is no correct client-side mapping, so
// rows here are informational only — no click-through to preview, no
// revert affordance.
const VersionHistoryList: React.FC<Props> = ({
  latestProjectSnapshotVersion,
  snapshots,
  isError,
}) => {
  const t = useT();
  // Sort by snapshotNumber, not timestamp: it is a monotonic per-room counter, so
  // it is the authoritative creation order and is unaffected by the zero
  // timestamp the server returns when one cannot be parsed.
  const sortedSnapshots = snapshots
    ? [...snapshots].sort((a, b) => b.snapshotNumber - a.snapshotNumber)
    : snapshots;

  return (
    <ScrollArea className="h-full w-full overflow-y-auto">
      {latestProjectSnapshotVersion && (
        <div className="flex items-center justify-between bg-primary px-2 py-2">
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

      {sortedSnapshots && sortedSnapshots.length > 0 ? (
        <div className="flex flex-col overflow-auto">
          {sortedSnapshots.map((snapshot) => (
            <div key={snapshot.snapshotNumber}>
              <div
                className="flex justify-between gap-2 px-2 py-2"
                style={{ height: "100%" }}>
                <p className="flex-2 self-center text-xs font-light dark:font-thin">
                  {snapshot.label && snapshot.label !== AUTO_SNAPSHOT_LABEL
                    ? snapshot.label
                    : formatDate(snapshot.timestamp)}
                </p>
                <div className="flex justify-end">
                  <p className="rounded border bg-border/15 p-1 text-xs font-thin dark:bg-primary/30">
                    <span className="font-light">
                      {" "}
                      {formatDate(snapshot.timestamp)}
                    </span>
                  </p>
                </div>
              </div>
              <div className="h-px bg-border" />
            </div>
          ))}
        </div>
      ) : isError ? (
        // Distinct from the empty state on purpose. A failed query used to render
        // "No versions yet", which tells the user their history does not exist
        // when in fact we could not load it — the worst possible reading on a
        // version-history panel.
        <p className="p-2 text-xs font-light select-none">
          {t("Could not load version history")}
        </p>
      ) : (
        <p className="p-2 text-xs font-light select-none">
          {t("No versions yet")}
        </p>
      )}
    </ScrollArea>
  );
};

export default VersionHistoryList;
