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
      title={`${t("Are you sure you want to revert to Version")} ${selectedVersion}?`}
      description={t("Test description here")}
      isOpen={!!selectedVersion}
      confirmDisabled={!selectedVersion}
      onClose={() => onDialogClose()}
      onConfirm={() => onDialogClose()}
    />
  );
};

export { VersionHistoryChangeDialog };
