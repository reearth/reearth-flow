import { Fragment } from "react/jsx-runtime";

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
    <Fragment key={version.id}>
      <div
        className={`flex cursor-pointer justify-between gap-2 rounded px-4 py-2 hover:bg-primary ${isSelected ? "bg-primary before:rounded-md before:border before:border-l-2 before:border-accent before:border-l-green-800/70 before:bg-primary before:opacity-100" : ""}`}
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
    </Fragment>
  );
};

export { Version };
