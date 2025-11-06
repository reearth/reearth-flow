import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  DialogFooter,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  debugRunStarted?: boolean;
  onDebugRunStart: () => Promise<void>;
  onDialogClose: () => void;
};

const DebugStartDialog: React.FC<Props> = ({
  debugRunStarted,
  onDebugRunStart,
  onDialogClose,
}) => {
  const t = useT();

  const handleDebugRunStart = async () => {
    await onDebugRunStart();
    onDialogClose();
  };

  return (
    <Dialog open={true} onOpenChange={onDialogClose}>
      <DialogContent size="xs">
        <DialogTitle>{t("Start Debug Run")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection>
            <p className="dark:font-light">
              {t(
                "Are you sure you want to start a debug run of this workflow?",
              )}
            </p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button variant="outline" size="sm" onClick={onDialogClose}>
            {t("Cancel")}
          </Button>
          <Button
            variant="default"
            size="sm"
            onClick={handleDebugRunStart}
            disabled={debugRunStarted}>
            {t("Start")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default DebugStartDialog;
