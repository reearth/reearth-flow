import ConfirmationDialog from "@flow/features/ConfirmationDialog";
import { useT } from "@flow/lib/i18n";

type Props = {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
};

const LegacyPortsDialog: React.FC<Props> = ({ isOpen, onClose, onConfirm }) => {
  const t = useT();

  return (
    <ConfirmationDialog
      title={t("Project update required")}
      description={t(
        'Action connection ports were renamed from "default" to "features" in the latest engine, and this project still uses the old names. Workflows will fail to run until it is updated. Continue to update all affected connections now — this change applies to the project for all collaborators and cannot be undone.',
      )}
      isOpen={isOpen}
      onClose={onClose}
      onConfirm={onConfirm}
    />
  );
};

export default LegacyPortsDialog;
