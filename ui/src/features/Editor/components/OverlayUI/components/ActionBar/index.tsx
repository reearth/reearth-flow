import {
  DotsThreeVertical,
  DownloadSimple,
  LetterCircleV,
  Play,
  RocketLaunch,
  Stop,
} from "@phosphor-icons/react";
import { memo, useState } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import { DeployDialog } from "./components";

const tooltipOffset = 6;

type Props = {
  allowedToDeploy: boolean;
  onWorkflowDeployment: (
    deploymentId?: string,
    description?: string,
  ) => Promise<void>;
};

const ActionBar: React.FC<Props> = ({
  allowedToDeploy,
  onWorkflowDeployment,
}) => {
  const t = useT();

  const [showDialog, setShowDialog] = useState(false);

  return (
    <>
      <div className="rounded-md border bg-secondary">
        <div className="flex rounded-md">
          <div className="flex align-middle">
            <IconButton
              className="rounded-l-[4px] rounded-r-none"
              tooltipText={t("Run project workflow")}
              tooltipOffset={tooltipOffset}
              icon={<Play weight="thin" />}
            />
            <IconButton
              className="rounded-none"
              tooltipText={t("Stop project workflow")}
              tooltipOffset={tooltipOffset}
              icon={<Stop weight="thin" />}
            />
            <IconButton
              className="rounded-none"
              tooltipText={t("Deploy project workflow")}
              tooltipOffset={tooltipOffset}
              icon={<RocketLaunch weight="thin" />}
              onClick={() => setShowDialog(true)}
            />
            {/* <IconButton
              className="rounded-[4px]"
              tooltipText={t("Download project workflow")}
              tooltipOffset={tooltipOffset}
              icon={<DownloadSimple weight="thin" />}
            /> */}
            <div className="border-l" />
            <DropdownMenu>
              <DropdownMenuTrigger>
                <IconButton
                  className="w-[20px] rounded-l-none rounded-r-[4px]"
                  tooltipText={t("Additional actions")}
                  tooltipOffset={tooltipOffset}
                  icon={<DotsThreeVertical />}
                />
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem className="flex gap-2" disabled>
                  <DownloadSimple weight="light" />
                  <p className="text-sm font-extralight">
                    {t("Export Project")}
                  </p>
                </DropdownMenuItem>
                <DropdownMenuItem className="flex gap-2">
                  <LetterCircleV weight="light" />
                  <p className="text-sm font-extralight">
                    {t("Version History")}
                  </p>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </div>
      {showDialog && (
        <DeployDialog
          allowedToDeploy={allowedToDeploy}
          setShowDialog={setShowDialog}
          onWorkflowDeployment={onWorkflowDeployment}
        />
      )}
    </>
  );
};

export default memo(ActionBar);
