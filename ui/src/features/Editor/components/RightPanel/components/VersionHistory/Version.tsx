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
        className={`flex cursor-pointer select-none
justify-between gap-2 rounded px-4 py-2 hover:bg-primary ${isSelected ? "border border-l-2 border-l-green-800/70 bg-primary" : ""}`}
        onClick={onClick}
        onDoubleClick={onDoubleClick}>
        <p className="flex-[2] text-xs font-thin">
          {formatDate(version.createdAt)}
        </p>
        <div className="flex justify-end">
          <p className="text-xs font-thin">
            {t("Version ")}
            <span className="font-light">{version.version}</span>
          </p>
        </div>
      </div>
      <div className="h-px bg-primary" />
    </div>
  );
};

export { Version };
