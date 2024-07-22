import { DownloadSimple, FloppyDiskBack, Play, Stop } from "@phosphor-icons/react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";

const tooltipOffset = 6;

const ActionBar = () => {
  const t = useT();

  return (
    <div className="absolute right-1 top-1">
      <div className="m-1 rounded-md border border-zinc-700 bg-background-800">
        <div className="flex rounded-md bg-zinc-900/50 p-1">
          <div className="flex align-middle">
            {/* <IconButton
            tooltipText={t("Publish workflow")}
            tooltipOffset={tooltipOffset}
            icon={<DoubleArrowRightIcon />}
          /> */}
            <IconButton
              tooltipText={t("Run workflow")}
              tooltipOffset={tooltipOffset}
              icon={<Play />}
            />
            <IconButton
              tooltipText={t("Stop workflow")}
              tooltipOffset={tooltipOffset}
              icon={<Stop />}
            />
            <IconButton
              tooltipText={t("Publish workflow")}
              tooltipOffset={tooltipOffset}
              icon={<FloppyDiskBack />}
            />
            <IconButton
              tooltipText={t("Download workflow")}
              tooltipOffset={tooltipOffset}
              icon={<DownloadSimple />}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export { ActionBar };
