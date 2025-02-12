import {
  DotsThreeVertical,
  DownloadSimple,
  LetterCircleV,
  Play,
  RocketLaunch,
  ShareFat,
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

import { DeployDialog, ShareDialog } from "./components";

const tooltipOffset = 6;

type Props = {
  allowedToDeploy: boolean;
  onWorkflowDeployment: (
    description: string,
    deploymentId?: string,
  ) => Promise<void>;
  onProjectShare: (share: boolean) => void;
  onRightPanelOpen: (content?: "version-history") => void;
};

const ActionBar: React.FC<Props> = ({
  allowedToDeploy,
  onWorkflowDeployment,
  onProjectShare,
  onRightPanelOpen,
}) => {
  const t = useT();

  const [showDeployDialog, setShowDeployDialog] = useState(false);
  const [showShareDialog, setShowShareDialog] = useState(false);

  const handleShowDeployDialog = () => {
    setShowShareDialog(false);
    setShowDeployDialog(true);
  };

  const handleShowShareDialog = () => {
    setShowDeployDialog(false);
    setShowShareDialog(true);
  };

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
              onClick={handleShowDeployDialog}
            />
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <IconButton
                  className="w-[25px] rounded-l-none rounded-r-[4px]"
                  tooltipText={t("Additional actions")}
                  tooltipOffset={tooltipOffset}
                  icon={<DotsThreeVertical />}
                />
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem
                  className="flex gap-2"
                  onClick={handleShowShareDialog}>
                  <ShareFat weight="light" />
                  <p className="text-sm font-extralight">
                    {t("Share Project")}
                  </p>
                </DropdownMenuItem>
                <DropdownMenuItem className="flex gap-2" disabled>
                  <DownloadSimple weight="light" />
                  <p className="text-sm font-extralight">
                    {t("Export Project")}
                  </p>
                </DropdownMenuItem>
                <DropdownMenuItem
                  className="flex gap-2"
                  onClick={() => onRightPanelOpen("version-history")}>
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
      {showDeployDialog && (
        <DeployDialog
          allowedToDeploy={allowedToDeploy}
          setShowDialog={setShowDeployDialog}
          onWorkflowDeployment={onWorkflowDeployment}
        />
      )}
      {showShareDialog && (
        <ShareDialog
          setShowDialog={setShowShareDialog}
          onProjectShare={onProjectShare}
        />
      )}
    </>
  );
};

export default memo(ActionBar);
