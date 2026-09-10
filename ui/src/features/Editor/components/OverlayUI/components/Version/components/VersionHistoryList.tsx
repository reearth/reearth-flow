import { ScrollArea } from "@flow/components";
import { useAwarenessEcho } from "@flow/features/Editor/editorContext";
import { useT } from "@flow/lib/i18n";
import { versionRowEchoKey } from "@flow/lib/yjs";
import { AUTO_SNAPSHOT_LABEL } from "@flow/types";
import type { NamedSnapshot, ProjectDocument } from "@flow/types";
import { awarenessEchoStripe, formatDate } from "@flow/utils";

/**
 * Split out so each row can subscribe to its own echo key — a hook cannot be
 * called inside the map callback.
 */
const SnapshotRow: React.FC<{
  snapshot: NamedSnapshot;
  isSelected: boolean;
  onSnapshotSelect?: (snapshotNumber: number) => void;
}> = ({ snapshot, isSelected, onSnapshotSelect }) => {
  const t = useT();
  const echoUsers = useAwarenessEcho(
    versionRowEchoKey(snapshot.snapshotNumber),
  );

  return (
    <div>
      <div
        className={`relative flex cursor-pointer justify-between gap-2 px-2 py-2 hover:bg-accent/50 ${isSelected ? "bg-accent" : ""}`}
        style={{ height: "100%", ...awarenessEchoStripe(echoUsers) }}
        role="button"
        tabIndex={0}
        aria-pressed={isSelected}
        onClick={() => onSnapshotSelect?.(snapshot.snapshotNumber)}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            onSnapshotSelect?.(snapshot.snapshotNumber);
          }
        }}>
        <div className="flex flex-col gap-1">
          <p className="flex-2 self-start text-xs font-light dark:font-thin">
            {snapshot.label && snapshot.label !== AUTO_SNAPSHOT_LABEL ? (
              snapshot.label
            ) : (
              <span className="opacity-70">{t("Autosaved")}</span>
            )}
          </p>
          <p className="text-xs font-thin">{formatDate(snapshot.timestamp)}</p>
        </div>
        <div className="flex items-center justify-end gap-2">
          {echoUsers.length > 0 && (
            <div className="flex items-center -space-x-2">
              {echoUsers.slice(0, 3).map((user) => (
                <div
                  key={user.userName}
                  title={user.userName}
                  className="flex size-5 items-center justify-center rounded-full ring-2 ring-secondary/20"
                  style={{ backgroundColor: user.color }}>
                  <span className="text-[10px] font-medium text-white select-none">
                    {user.userName.charAt(0).toUpperCase()}
                  </span>
                </div>
              ))}
            </div>
          )}
          <p className="h-fit rounded border bg-border/15 p-1 text-xs font-thin dark:bg-primary/30">
            <span className="font-light">
              {t("Snapshot ")}
              {snapshot.snapshotNumber}
            </span>
          </p>
        </div>
      </div>
      <div className="h-px bg-border" />
    </div>
  );
};

type Props = {
  latestProjectSnapshotVersion?: ProjectDocument;
  snapshots?: NamedSnapshot[];
  isError?: boolean;
  selectedSnapshotNumber?: number | null;
  onSnapshotSelect?: (snapshotNumber: number) => void;
};

const VersionHistoryList: React.FC<Props> = ({
  latestProjectSnapshotVersion,
  snapshots,
  isError,
  selectedSnapshotNumber,
  onSnapshotSelect,
}) => {
  const t = useT();
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
            <SnapshotRow
              key={snapshot.snapshotNumber}
              snapshot={snapshot}
              isSelected={selectedSnapshotNumber === snapshot.snapshotNumber}
              onSnapshotSelect={onSnapshotSelect}
            />
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
