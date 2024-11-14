import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  title: string;
  description: string;
  isOpen: boolean;
  confirmDisabled?: boolean;
  onClose: () => void;
  onConfirm: () => void;
};

const ConfirmationDialog: React.FC<Props> = ({
  title,
  description,
  isOpen,
  confirmDisabled,
  onClose,
  onConfirm,
}) => {
  const t = useT();
  return (
    <AlertDialog open={isOpen}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{title}</AlertDialogTitle>
          <AlertDialogDescription>{description}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={onClose}>{t("Cancel")}</AlertDialogCancel>
          <AlertDialogAction disabled={!!confirmDisabled} onClick={onConfirm}>
            {t("Continue")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
};

export default ConfirmationDialog;
