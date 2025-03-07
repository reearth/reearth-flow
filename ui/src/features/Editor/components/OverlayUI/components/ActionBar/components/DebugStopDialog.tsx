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
  onDebugRunStop: () => Promise<void>;
  onDialogClose: () => void;
};

const DebugStopDialog: React.FC<Props> = ({
  onDebugRunStop,
  onDialogClose,
}) => {
  const t = useT();

  const handleDebugRunStop = async () => {
    await onDebugRunStop();
    onDialogClose();
  };

  return (
    <Dialog open={true} onOpenChange={onDialogClose}>
      <DialogContent size="xs">
        <DialogTitle>{t("Stop Workflow")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection>
            <p className="dark:font-light">
              {t("Are you sure you want to stop the workflow's debug run?")}
            </p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button variant="destructive" size="sm" onClick={handleDebugRunStop}>
            {t("Stop")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default DebugStopDialog;
