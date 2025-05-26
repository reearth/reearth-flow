import {
  DotsThreeVertical,
  Export,
  LetterCircleV,
  PaperPlaneTilt,
  Rocket,
} from "@phosphor-icons/react";
import { memo } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import { DeployDialog, ShareDialog } from "./components";
import useHooks from "./hooks";

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
  const {
    showDialog,
    handleShowDeployDialog,
    handleShowShareDialog,
    handleDialogClose,
    handleProjectExport,
  } = useHooks();

  return (
    <>
      <div className="flex rounded-md bg-secondary">
        <div className="flex align-middle gap-2">
          <IconButton
            tooltipText={t("Deploy project's workflow")}
            tooltipOffset={tooltipOffset}
            icon={<Rocket weight="thin" size={18} />}
            onClick={handleShowDeployDialog}
          />
          <IconButton
            tooltipText={t("Share Project")}
            tooltipOffset={tooltipOffset}
            icon={<PaperPlaneTilt weight="thin" size={18} />}
            onClick={handleShowShareDialog}
          />
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <IconButton
                className="w-[25px]"
                tooltipText={t("Additional actions")}
                tooltipOffset={tooltipOffset}
                icon={<DotsThreeVertical size={18} />}
              />
            </DropdownMenuTrigger>
            <DropdownMenuContent
              className="flex flex-col gap-2"
              align="end"
              sideOffset={10}
              alignOffset={2}>
              <DropdownMenuItem
                className="flex justify-between gap-4"
                onClick={handleProjectExport}>
                <p>{t("Export Project")}</p>
                <Export weight="thin" size={18} />
              </DropdownMenuItem>
              <DropdownMenuItem
                className="flex justify-between gap-4"
                onClick={() => onRightPanelOpen("version-history")}>
                <p>{t("Version History")}</p>
                <LetterCircleV weight="thin" size={18} />
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
      {showDialog === "deploy" && (
        <DeployDialog
          allowedToDeploy={allowedToDeploy}
          onDialogClose={handleDialogClose}
          onWorkflowDeployment={onWorkflowDeployment}
        />
      )}
      {showDialog === "share" && (
        <ShareDialog
          onDialogClose={handleDialogClose}
          onProjectShare={onProjectShare}
        />
      )}
    </>
  );
};

export default memo(ActionBar);
