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

  const [selectedVersion, setSelectedVersion] = useState<string | null>(null);

  const [openVersionChangeDialog, setOpenVersionChangeDialog] = useState(false);

  const currentVersion = versionHistory.length > 0 ? versionHistory[0] : null;
  const olderVersions = versionHistory.slice(1);
  const handleVersionClick = (id: string, version: string) => {
    setSelectedVersionId(id);
    setSelectedVersion(version);
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
            <p className="rounded border bg-logo/30 p-1 text-xs font-thin">
              <span className="font-light">
                {" "}
                {t("Version ")}
                {currentVersion.version}
              </span>
            </p>
          </div>
        )}
        <div className="flex flex-col overflow-auto">
          {olderVersions.map((history) => (
            <Version
              key={history.id}
              version={history}
              isSelected={history.id === selectedVersionId}
              onClick={() => handleVersionClick(history.id, history.version)}
              onDoubleClick={handleDoubleClick}
            />
          ))}
          <div className="pb-8" />
        </div>
      </ScrollArea>
      {openVersionChangeDialog && selectedVersion && (
        <VersionHistoryChangeDialog
          selectedVersion={selectedVersion}
          onDialogClose={() => setOpenVersionChangeDialog(false)}
        />
      )}
    </div>
  );
};

export { VersionHistoryList };
