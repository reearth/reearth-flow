import { useT } from "@flow/lib/i18n";
import { formatDate } from "@flow/utils";

type VersionProps = {
  version: {
    id: string;
    version: string;
    createdAt: string;
    isCurrent?: boolean;
  };
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
    <div key={version.id}>
      <div
        className={`flex cursor-pointer select-none justify-between gap-2 rounded px-4 py-2 ${isSelected ? "relative before:absolute before:inset-y-0 before:left-0 before:right-1 before:-z-10 before:rounded-md before:border before:border-l-2 before:border-accent before:border-l-logo/30 before:bg-primary before:opacity-100" : "hover:bg-primary"}`}
        onClick={onClick}
        onDoubleClick={onDoubleClick}
        style={{ height: "100%" }}>
        <p className="flex-[2] self-center text-xs font-thin">
          {formatDate(version.createdAt)}
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
