import ConfirmationDialog from "@flow/features/ConfirmationDialog";
import { useT } from "@flow/lib/i18n";

type Props = {
  selectedProjectSnapshotVersion: number;
  onDialogClose: () => void;
  onRollbackProject: () => void;
};

const VersionHistoryChangeDialog: React.FC<Props> = ({
  selectedProjectSnapshotVersion,
  onDialogClose,
  onRollbackProject,
}) => {
  const t = useT();
  return (
    <ConfirmationDialog
      title={t("Are you sure you want to revert to this version?")}
      description={t(
        "By clicking continue you will be reverting to version {{version}}.",
        { version: selectedProjectSnapshotVersion },
      )}
      isOpen={!!selectedProjectSnapshotVersion}
      confirmDisabled={!selectedProjectSnapshotVersion}
      onClose={() => onDialogClose()}
      onConfirm={() => onRollbackProject()}
    />
  );
};

export { VersionHistoryChangeDialog };
