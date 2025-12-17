import ConfirmationDialog from "@flow/features/ConfirmationDialog";
import { useT } from "@flow/lib/i18n";

type Props = {
  isDeleteDialogOpen: boolean;
  setIsDeleteDialogOpen: (value: boolean) => void;
  onWorkerConfigDelete: () => Promise<void>;
};

const WorkerConfigDeletionDialog: React.FC<Props> = ({
  isDeleteDialogOpen,
  setIsDeleteDialogOpen,
  onWorkerConfigDelete,
}) => {
  const t = useT();

  return (
    <ConfirmationDialog
      title={t("Are you absolutely sure?")}
      description="This action will delete the worker configuration and reset it
                  to default values. This action cannot be undone."
      isOpen={!!isDeleteDialogOpen}
      onClose={() => setIsDeleteDialogOpen(false)}
      onConfirm={async () => {
        await onWorkerConfigDelete();
      }}
    />
  );
};

export default WorkerConfigDeletionDialog;
