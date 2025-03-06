import {
  DotsThreeVertical,
  Export,
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
import { useProjectExport } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";

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

  const [currentProject] = useCurrentProject();

  const { handleProjectExport } = useProjectExport(currentProject);

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
                  className="flex justify-between gap-4"
                  onClick={handleShowShareDialog}>
                  <p>{t("Share Project")}</p>
                  <ShareFat weight="light" />
                </DropdownMenuItem>
                <DropdownMenuItem
                  className="flex justify-between gap-4"
                  onClick={handleProjectExport}>
                  <p>{t("Export Project")}</p>
                  <Export weight="light" />
                </DropdownMenuItem>
                <DropdownMenuItem
                  className="flex justify-between gap-4"
                  onClick={() => onRightPanelOpen("version-history")}>
                  <p>{t("Version History")}</p>
                  <LetterCircleV weight="light" />
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
