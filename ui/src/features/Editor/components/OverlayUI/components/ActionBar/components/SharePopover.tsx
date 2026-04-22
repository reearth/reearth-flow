import { PaperclipIcon, PaperPlaneTiltIcon } from "@phosphor-icons/react";

import { Button, Switch } from "@flow/components";
import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";

type Props = {
  sharingUrl?: string;
  onProjectShare: (share: boolean) => void;
};

const SharePopover: React.FC<Props> = ({ sharingUrl, onProjectShare }) => {
  const t = useT();
  const { toast } = useToast();

  const isSharing = !!sharingUrl;

  const handleCopyUrl = () => {
    navigator.clipboard
      .writeText(sharingUrl || "")
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
    <div className="flex flex-col gap-2 p-4 pt-2">
      <div className="flex justify-between gap-2">
        <h4 className="text-md flex items-center gap-2 self-center rounded-t-lg leading-none tracking-tight dark:font-thin">
          <PaperPlaneTiltIcon weight="thin" size={16} />
          {t("Share Project")}
        </h4>
        <Button
          className="flex gap-2"
          variant="outline"
          disabled={!sharingUrl}
          onClick={handleCopyUrl}>
          <PaperclipIcon weight="thin" />
          <p className="text-xs dark:font-light">{t("Copy URL")}</p>
        </Button>
      </div>
      <div className="flex flex-col gap-4">
        <p className="text-xs dark:font-light">
          {t(
            "Share your project's workflow with anyone with the URL. This is limited access to reading the contents of the canvas.",
          )}
        </p>
        <div className="flex items-center gap-2">
          <Switch checked={isSharing} onCheckedChange={onProjectShare} />
          <span className="text-sm dark:font-light">{t("Sharing")}</span>
        </div>
      </div>
    </div>
  );
};

export default SharePopover;
