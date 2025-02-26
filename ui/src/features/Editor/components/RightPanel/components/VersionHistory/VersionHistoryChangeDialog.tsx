import ConfirmationDialog from "@flow/features/ConfirmationDialog";
import { useT } from "@flow/lib/i18n";

type Props = {
  selectedVersion: string;
  onDialogClose: () => void;
};

const VersionHistoryChangeDialog: React.FC<Props> = ({
  selectedVersion,
  onDialogClose,
}) => {
  const t = useT();
  return (
    <ConfirmationDialog
      title={t("Are you absolutely sure you want to change version?")}
      description={t("Test description here")}
      isOpen={!!selectedVersion}
      confirmDisabled={!selectedVersion}
      onClose={() => onDialogClose()}
      onConfirm={() => onDialogClose()}
    />
  );
};

export { VersionHistoryChangeDialog };
