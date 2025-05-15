import { Paperclip } from "@phosphor-icons/react";
import { useCallback, useState } from "react";

import { Button, Switch } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";

type Props = {
  onProjectShare: (share: boolean) => void;
  onDialogClose: () => void;
};

type SharingState = "sharing" | "notSharing";

const ShareDialog: React.FC<Props> = ({ onProjectShare, onDialogClose }) => {
  const t = useT();
  const [currentProject] = useCurrentProject();

  const BASE_URL = window.location.origin;
  const sharedToken = currentProject?.sharedToken;
  const sharedUrl = sharedToken
    ? BASE_URL + "/shared/" + sharedToken
    : undefined;

  const [hasBeenEdited, setHasBeenEdited] = useState(false);
  const [isSharing, setIsSharing] = useState<SharingState>(
    currentProject?.sharedToken ? "sharing" : "notSharing",
  );

  const [isSwitchOn, setIsSwitchOn] = useState<boolean>(
    isSharing === "sharing",
  );

  const handleSharingChange = useCallback((checked: boolean) => {
    const share = checked ? "sharing" : "notSharing";
    setIsSharing(share);
    setIsSwitchOn(checked);
    setHasBeenEdited(true);
  }, []);

  const handleProjectShare = useCallback(() => {
    onProjectShare(isSharing === "sharing");
    onDialogClose();
  }, [isSharing, onProjectShare, onDialogClose]);
  return (
    <div>
      <div className="flex flex-col gap-2">
        <div className="flex gap-2 justify-between border-b py-2">
          <h4 className="text-xl dark:font-thin leading-none tracking-tight py-2 rounded-t-lg">
            {t("Share Project")}
          </h4>
          <Button
            className="flex gap-2"
            variant="default"
            disabled={!isSwitchOn}
            onClick={() => navigator.clipboard.writeText(sharedUrl || "")}>
            <Paperclip weight="thin" />
            <p className="text-xs dark:font-light">{t("Copy URL")}</p>
          </Button>
        </div>

        <div>
          <div className="flex flex-col gap-2">
            <p className="text-sm">
              {t(
                "Share your project's workflow with anyone with the URL. This is limited access to reading the contents of the canvas.",
              )}
            </p>
            <div className="flex items-center gap-2">
              <Switch
                checked={isSwitchOn}
                onCheckedChange={handleSharingChange}
              />
              <span className="text-sm">{t("Sharing")}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ShareDialog;
