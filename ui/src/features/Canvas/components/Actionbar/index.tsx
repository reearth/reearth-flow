import { DownloadIcon, Link2Icon, PlayIcon, StopIcon } from "@radix-ui/react-icons";

import { IconButton } from "@flow/components";
import { useT } from "@flow/providers";

const tooltipOffset = 6;

export default function ActionBar() {
  const t = useT();

  return (
    <div className="absolute top-1 right-1">
      <div className="bg-zinc-800 rounded-md m-1 border border-zinc-700">
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
              icon={<PlayIcon />}
            />
            <IconButton
              tooltipText={t("Stop workflow")}
              tooltipOffset={tooltipOffset}
              icon={<StopIcon />}
            />
            <IconButton
              tooltipText={t("Publish workflow")}
              tooltipOffset={tooltipOffset}
              icon={<Link2Icon />}
            />
            <IconButton
              tooltipText={t("Download workflow")}
              tooltipOffset={tooltipOffset}
              icon={<DownloadIcon />}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
