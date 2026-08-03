import { ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { NamedSnapshot, ProjectDocument } from "@flow/types";
import { formatDate } from "@flow/utils";

type Props = {
  latestProjectSnapshotVersion?: ProjectDocument;
  snapshots?: NamedSnapshot[];
  onSnapshotSelect: (id: number) => void;
  selectedProjectSnapshotVersion?: number | null;
};

const VersionHistoryList: React.FC<Props> = ({
  latestProjectSnapshotVersion,
  snapshots,
  selectedProjectSnapshotVersion,
  onSnapshotSelect,
}) => {
  const t = useT();
  // Snapshots are already distinct from the live head (unlike the raw update
  // log this list used to render), so there is no head entry to filter out
  // here. Sort defensively since the backend does not guarantee ordering.
  const sortedSnapshots = snapshots
    ? [...snapshots].sort(
        (a, b) =>
          new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime(),
      )
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
            <div key={snapshot.id}>
              <div
                role="button"
                tabIndex={0}
                className={`flex cursor-pointer justify-between gap-2 px-2 py-2 select-none ${snapshot.id === selectedProjectSnapshotVersion ? "bg-border/40 dark:bg-primary" : "hover:bg-border/40 dark:hover:bg-primary"}`}
                onClick={() => onSnapshotSelect(snapshot.id)}
                style={{ height: "100%" }}>
                <p className="flex-2 self-center text-xs font-light dark:font-thin">
                  {snapshot.label || formatDate(snapshot.timestamp)}
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
      ) : (
        <p className="p-2 text-xs font-light select-none">
          {t("No versions yet")}
        </p>
      )}
    </ScrollArea>
  );
};

export default VersionHistoryList;
