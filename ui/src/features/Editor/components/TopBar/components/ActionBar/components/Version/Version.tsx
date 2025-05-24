import { useT } from "@flow/lib/i18n";
import type { ProjectSnapshotMeta } from "@flow/types";
import { formatDate } from "@flow/utils";

type VersionProps = {
  version: ProjectSnapshotMeta;
  isSelected: boolean;
  onClick: () => void;
  onDoubleClick: () => void;
};

const Version: React.FC<VersionProps> = ({
  version,
  isSelected,
  onClick,
  onDoubleClick,
}) => {
  const t = useT();

  return (
    <div>
      <div
        className={`flex cursor-pointer select-none justify-between gap-2 px-2 py-2 ${isSelected ? "bg-primary" : "hover:bg-primary"}`}
        onClick={onClick}
        onDoubleClick={onDoubleClick}
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
  );
};

export { Version };
