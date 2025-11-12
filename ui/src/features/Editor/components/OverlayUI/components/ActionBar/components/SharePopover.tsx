import { PaperclipIcon } from "@phosphor-icons/react";
import { debounce } from "lodash-es";
import { useEffect, useRef, useState } from "react";

import { Button, Switch } from "@flow/components";
import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";

type Props = {
  onProjectShare: (share: boolean) => void;
};

const SharePopover: React.FC<Props> = ({ onProjectShare }) => {
  const t = useT();
  const { toast } = useToast();
  const [currentProject] = useCurrentProject();
  const BASE_URL = window.location.origin;
  const sharedToken = currentProject?.sharedToken;
  const sharedUrl = sharedToken
    ? BASE_URL + "/shared/" + sharedToken
    : undefined;

  const [isSharing, setIsSharing] = useState<boolean>(
    !!currentProject?.sharedToken,
  );

  useEffect(() => {
    setIsSharing(!!currentProject?.sharedToken);
  }, [currentProject?.sharedToken]);

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
      onProjectShare(checked);
    },
    2000,
  );

  const handleSharingChange = (checked: boolean) => {
    setIsSharing(checked);
    debouncedHandleSharingChange(checked);
  };

  const handleCopyUrl = () => {
    navigator.clipboard
      .writeText(sharedUrl || "")
      .then(() => {
        toast({
          title: t("URL Copied."),
          description: t("URL was successfully copied to your clipboard."),
        });
      })
      .catch(() => {
        toast({
          title: t("URL Copy Failed"),
          description: t("Failed to copy URL to clipboard."),
          variant: "destructive",
        });
      });
  };

  return (
    <div className="flex flex-col gap-0.5">
      <div className="flex justify-between gap-2 p-4">
        <h4 className="text-md self-center rounded-t-lg leading-none tracking-tight dark:font-thin">
          {t("Share Project")}
        </h4>
        <Button
          className="flex gap-2"
          variant="outline"
          disabled={!currentProject?.sharedToken}
          onClick={handleCopyUrl}>
          <PaperclipIcon weight="thin" />
          <p className="text-xs dark:font-light">{t("Copy URL")}</p>
        </Button>
      </div>
      <div className="flex flex-col gap-3 p-4 pt-0">
        <p className="text-xs dark:font-light">
          {t(
            "Share your project's workflow with anyone with the URL. This is limited access to reading the contents of the canvas.",
          )}
        </p>
        <div className="flex items-center gap-2">
          <Switch checked={isSharing} onCheckedChange={handleSharingChange} />
          <span className="text-sm dark:font-light">{t("Sharing")}</span>
        </div>
      </div>
    </div>
  );
};

export default SharePopover;
