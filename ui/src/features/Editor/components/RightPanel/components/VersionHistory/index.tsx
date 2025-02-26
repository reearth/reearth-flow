import { useState } from "react";

import { ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { VersionHistory } from "@flow/mock_data/versionHistoryData";
import { formatDate } from "@flow/utils";

import { Version } from "./Version";
import { VersionHistoryChangeDialog } from "./VersionHistoryChangeDialog";

type Props = {
  versionHistory: VersionHistory[];
};

const VersionHistoryList: React.FC<Props> = ({ versionHistory }) => {
  const t = useT();
  const [selectedVersionId, setSelectedVersionId] = useState<string | null>(
    null,
  );

  const [openVersionChangeDialog, setOpenVersionChangeDialog] = useState(false);

  const currentVersion = versionHistory.length > 0 ? versionHistory[0] : null;
  const olderVersions = versionHistory.slice(1);
  const handleVersionClick = (id: string) => {
    setSelectedVersionId(id);
  };

  const handleDoubleClick = () => {
    setOpenVersionChangeDialog(true);
  };

  return (
    <div className="flex h-full flex-col overflow-auto">
      <ScrollArea>
        {currentVersion && (
          <div className="flex items-center justify-between rounded bg-primary p-1 px-4">
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
        <div className="flex flex-col overflow-auto">
          {olderVersions.map((history) => (
            <Version
              key={history.id}
              version={history}
              isSelected={history.id === selectedVersionId}
              onClick={() => handleVersionClick(history.id)}
              onDoubleClick={handleDoubleClick}
            />
          ))}
          <div className="pb-6" />
        </div>
      </ScrollArea>
      {openVersionChangeDialog && selectedVersionId && (
        <VersionHistoryChangeDialog
          selectedVersion={selectedVersionId}
          onDialogClose={() => setOpenVersionChangeDialog(false)}
        />
      )}
    </div>
  );
};

export { VersionHistoryList };
