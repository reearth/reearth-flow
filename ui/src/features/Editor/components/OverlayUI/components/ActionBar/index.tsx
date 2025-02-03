import {
  DownloadSimple,
  Play,
  RocketLaunch,
  Stop,
} from "@phosphor-icons/react";
import { memo, useState } from "react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";

import { DeployDialog } from "./components";

const tooltipOffset = 6;

type Props = {
  allowedToDeploy: boolean;
  onWorkflowDeployment: (
    description: string,
    deploymentId?: string,
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
      <div className="absolute right-1 top-1">
        <div className="m-1 rounded-md border bg-secondary">
          <div className="flex rounded-md">
            <div className="flex align-middle">
              <IconButton
                className="rounded-[4px]"
                tooltipText={t("Run project workflow")}
                tooltipOffset={tooltipOffset}
                icon={<Play weight="thin" />}
              />
              <IconButton
                className="rounded-[4px]"
                tooltipText={t("Stop project workflow")}
                tooltipOffset={tooltipOffset}
                icon={<Stop weight="thin" />}
              />
              <IconButton
                className="rounded-[4px]"
                tooltipText={t("Deploy project workflow")}
                tooltipOffset={tooltipOffset}
                icon={<RocketLaunch weight="thin" />}
                onClick={() => setShowDialog(true)}
              />
              <IconButton
                className="rounded-[4px]"
                tooltipText={t("Download project workflow")}
                tooltipOffset={tooltipOffset}
                icon={<DownloadSimple weight="thin" />}
              />
            </div>
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
