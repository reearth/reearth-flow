import { X } from "@phosphor-icons/react";
import { memo } from "react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { mockVersionHistory } from "@flow/mock_data/versionHistoryData";

import { VersionHistoryList } from "./components";

type Props = {
  contentType?: "version-history";
  onClose: () => void;
};

const RightPanel: React.FC<Props> = ({ contentType, onClose }) => {
  const t = useT();
  // TODO: Hook up gql, correct types and remove mock data
  const versionHistory = [...mockVersionHistory];

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
      <div className="flex items-center border-b">
        <IconButton
          className="m-1 size-[30px] shrink-0"
          icon={<X className="size-[18px]" weight="thin" />}
          onClick={onClose}
        />
        <div className="grow p-2 text-center">
          <p className="text-lg dark:font-thin">{t("Version History")}</p>
        </div>
      </div>
      {contentType === "version-history" && (
        <VersionHistoryList versionHistory={versionHistory} />
      )}
    </div>
  );
};

export default memo(RightPanel);
