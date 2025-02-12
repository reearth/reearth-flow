import { useCallback, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  DialogFooter,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  DialogDescription,
  Label,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";

type Props = {
  onProjectShare: (share: boolean) => void;
  setShowDialog: (show: boolean) => void;
};

type SharingState = "sharing" | "notSharing";

const ShareDialog: React.FC<Props> = ({ onProjectShare, setShowDialog }) => {
  const t = useT();
  const [currentProject] = useCurrentProject();

  const [hasBeenEdited, setHasBeenEdited] = useState(false);
  const [isSharing, setIsSharing] = useState<SharingState>("notSharing");

  const handleSharingChange = useCallback((share: SharingState) => {
    setIsSharing(share);
    setHasBeenEdited(true);
  }, []);

  const sharingLabels = {
    sharing: t("Sharing"),
    notSharing: t("Not Sharing"),
  };

  const handleProjectShare = useCallback(() => {
    onProjectShare(isSharing === "sharing");
    setShowDialog(false);
  }, [isSharing, onProjectShare, setShowDialog]);

  return (
    <Dialog open={true} onOpenChange={() => setShowDialog(false)}>
      <DialogContent size="sm">
        <DialogTitle>{t("Share Project")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection>
            <DialogDescription>
              {t(
                "Share your project's workflow with anyone with the URL. This is limited access to reading the contents of the canvas.",
              )}
            </DialogDescription>
            <Select onValueChange={handleSharingChange}>
              <SelectTrigger>
                <SelectValue
                  placeholder={
                    isSharing === "sharing"
                      ? t("Currently sharing")
                      : t("Currently not sharing")
                  }
                />
              </SelectTrigger>
              <SelectContent>
                {Object.entries(sharingLabels).map(([key, label]) => (
                  <SelectItem key={key} value={key}>
                    {label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </DialogContentSection>
          <DialogContentSection className="break-all">
            <Label>{t("URL: ")}</Label>
            <p className="text-wrap font-thin">
              https://someUrl.reearth.flow.io/preview/project/alskdfjasldfkjasldkfj
            </p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button disabled={hasBeenEdited} onClick={handleProjectShare}>
            {t("Submit")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default ShareDialog;
