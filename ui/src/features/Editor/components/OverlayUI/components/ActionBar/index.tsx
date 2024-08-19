import {
  DownloadSimple,
  Play,
  RocketLaunch,
  Stop,
} from "@phosphor-icons/react";
import { memo } from "react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";

const tooltipOffset = 6;

const ActionBar = () => {
  const t = useT();

  return (
    <div className="absolute right-1 top-1">
      <div className="m-1 rounded-md border bg-secondary">
        <div className="flex rounded-md">
          <div className="flex align-middle">
            {/* <IconButton
            tooltipText={t("Publish workflow")}
            tooltipOffset={tooltipOffset}
            icon={<DoubleArrowRightIcon />}
          /> */}
            <IconButton
              className="rounded-[4px]"
              tooltipText={t("Run workflow")}
              tooltipOffset={tooltipOffset}
              icon={<Play weight="thin" />}
            />
            <IconButton
              className="rounded-[4px]"
              tooltipText={t("Stop workflow")}
              tooltipOffset={tooltipOffset}
              icon={<Stop weight="thin" />}
            />
            <IconButton
              className="rounded-[4px]"
              tooltipText={t("Deploy workflow")}
              tooltipOffset={tooltipOffset}
              icon={<RocketLaunch weight="thin" />}
            />
            <IconButton
              className="rounded-[4px]"
              tooltipText={t("Download workflow")}
              tooltipOffset={tooltipOffset}
              icon={<DownloadSimple weight="thin" />}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default memo(ActionBar);
