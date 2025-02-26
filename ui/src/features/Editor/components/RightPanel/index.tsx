import { LetterCircleV, X } from "@phosphor-icons/react";
import { Fragment, memo, useState } from "react";

import { IconButton, ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { mockVersionHistory } from "@flow/mock_data/versionHistoryData";
import { formatDate } from "@flow/utils";

import { VersionHistoryChangeDialog } from "./VersionHistoryChangeDialog";

type Props = {
  contentType?: "version-history";
  onClose: () => void;
};

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

const RightPanel: React.FC<Props> = ({ contentType, onClose }) => {
  const t = useT();
  const [selectedVersionId, setSelectedVersionId] = useState<string | null>(
    null,
  );

  const [openVersionChangeDialog, setOpenVersionChangeDialog] = useState(false);

  const versionHistory = [...mockVersionHistory];

  const currentVersion = versionHistory.shift();

  const handleVersionClick = (id: string) => {
    setSelectedVersionId(id);
  };

  const handleDoubleClick = () => {
    setOpenVersionChangeDialog(true);
  };

  return (
    <div
      id="right-panel"
      className="fixed right-0 z-10 flex h-full w-[300px] flex-col border-l bg-background transition-all"
      style={{
        transform: `translateX(${contentType ? "0" : "100%"})`,
        transitionDuration: contentType ? "500ms" : "300ms",
        transitionProperty: "transform",
        transitionTimingFunction: "cubic-bezier(0.4, 0, 0.2, 1)",
      }}>
      <div className="flex justify-between border-b">
        <IconButton
          className="m-1 size-[30px] shrink-0"
          icon={<X className="size-[18px]" weight="thin" />}
          onClick={onClose}
        />
        <div className="flex items-center gap-1 p-2">
          <LetterCircleV weight="light" />
          <p className="text-xs font-extralight">{t("Version")}</p>
        </div>
      </div>
      {contentType === "version-history" && (
        <div className="flex h-full flex-col overflow-auto">
          <ScrollArea>
            {currentVersion && (
              <div className="flex items-center justify-between rounded bg-primary p-1">
                <div>
                  <p className="text-sm font-light">{t("Current Version")}</p>
                  <p className="flex-[2] text-xs font-thin">
                    {formatDate(currentVersion.createdAt)}
                  </p>
                </div>
                <p className="text-xs font-thin">
                  {t("Version ")}
                  <span className="font-light">{currentVersion.version}</span>
                </p>
              </div>
            )}
            {versionHistory.map((history) => (
              <Version
                key={history.id}
                version={history}
                isSelected={history.id === selectedVersionId}
                onClick={() => handleVersionClick(history.id)}
                onDoubleClick={handleDoubleClick}
              />
            ))}
          </ScrollArea>
        </div>
      )}
      {openVersionChangeDialog && selectedVersionId && (
        <VersionHistoryChangeDialog
          selectedVersion={selectedVersionId}
          onDialogClose={() => setOpenVersionChangeDialog(false)}
        />
      )}
    </div>
  );
};

export default memo(RightPanel);

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
        className={`flex cursor-pointer justify-between gap-2 rounded px-1 py-2 hover:bg-primary ${isSelected ? "bg-primary before:rounded-md before:border before:border-l-2 before:border-accent before:border-l-green-800/70 before:bg-primary before:opacity-100" : ""}`}
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
