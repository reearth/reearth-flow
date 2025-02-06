import { LetterCircleV, X } from "@phosphor-icons/react";
import { Fragment, memo } from "react";

import { IconButton, ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { mockVersionHistory } from "@flow/mock_data/versionHistoryData";
import { formatDate } from "@flow/utils";

type Props = {
  contentType?: "version-history";
  onClose: () => void;
};

const RightPanel: React.FC<Props> = ({ contentType, onClose }) => {
  const t = useT();

  const versionHistory = [...mockVersionHistory];

  const currentVersion = versionHistory.shift();

  return (
    <div
      id="right-panel"
      className="fixed right-0 flex h-full w-[300px] flex-col border-l bg-background transition-all"
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
          <p className="text-xs font-extralight">Version History</p>
        </div>
      </div>
      {contentType === "version-history" && (
        <div className="flex h-full flex-col overflow-auto">
          <ScrollArea>
            {currentVersion && (
              <div className="m-1 flex items-center justify-between rounded bg-primary p-1">
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
              <Fragment key={history.id}>
                <div className="m-1 flex cursor-pointer justify-between gap-2 rounded px-1 py-2 hover:bg-primary">
                  <p className="flex-[2] text-xs font-thin">
                    {formatDate(history.createdAt)}
                  </p>
                  <div className="flex justify-end">
                    <p className="text-xs font-thin">
                      {t("Version ")}
                      <span className="font-light">{history.version}</span>
                    </p>
                  </div>
                </div>
                <div className="h-px bg-primary" />
              </Fragment>
            ))}
          </ScrollArea>
        </div>
      )}
    </div>
  );
};

export default memo(RightPanel);
