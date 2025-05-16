import { Paperclip } from "@phosphor-icons/react";
import { debounce } from "lodash-es";
import { useEffect, useRef, useState } from "react";

import { Button, Switch } from "@flow/components";
import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";

type Props = {
  onProjectShare: (share: boolean) => void;
  onDialogClose: () => void;
};

type SharingState = "sharing" | "notSharing";

const SharePopover: React.FC<Props> = ({ onProjectShare }) => {
  const t = useT();
  const { toast } = useToast();
  const [currentProject] = useCurrentProject();
  const BASE_URL = window.location.origin;
  const sharedToken = currentProject?.sharedToken;
  const sharedUrl = sharedToken
    ? BASE_URL + "/shared/" + sharedToken
    : undefined;

  const [isSharing, setIsSharing] = useState<SharingState>(
    currentProject?.sharedToken ? "sharing" : "notSharing",
  );

  const [isSwitchOn, setIsSwitchOn] = useState<boolean>(
    isSharing === "sharing",
  );
  const useDebouncedCallback = (
    callback: (checked: boolean) => void,
    delay: number,
  ) => {
    const callbackRef = useRef(callback);

    useEffect(() => {
      callbackRef.current = callback;
    }, [callback]);

    return useRef(
      debounce((...args: [boolean]) => callbackRef.current(...args), delay),
    ).current;
  };

  const debouncedHandleSharingChange = useDebouncedCallback(
    (checked: boolean) => {
      const share = checked ? "sharing" : "notSharing";
      setIsSharing(share);
      setIsSwitchOn(checked);
      onProjectShare(share === "sharing");
    },
    200,
  );

  const handleCopyUrl = () => {
    navigator.clipboard.writeText(sharedUrl || "");
    toast({
      title: t("URL copied."),
      description: t("URL was successfully copied to your clipboard."),
    });
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="flex gap-2 justify-between border-b pb-2">
        <h4 className="text-md self-center dark:font-thin leading-none tracking-tight rounded-t-lg">
          {t("Share Project")}
        </h4>
        <Button
          className="flex gap-2"
          variant="default"
          disabled={!isSwitchOn && !sharedUrl}
          onClick={handleCopyUrl}>
          <Paperclip weight="thin" />
          <p className="text-xs dark:font-light">{t("Copy URL")}</p>
        </Button>
      </div>
      <div className="flex flex-col gap-4">
        <p className="text-xs">
          {t(
            "Share your project's workflow with anyone with the URL. This is limited access to reading the contents of the canvas.",
          )}
        </p>
        <div className="flex items-center gap-2">
          <Switch
            checked={isSwitchOn}
            onCheckedChange={debouncedHandleSharingChange}
          />
          <span className="text-sm">{t("Sharing")}</span>
        </div>
      </div>
    </div>
  );
};

export default SharePopover;
