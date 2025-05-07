import ConfirmationDialog from "@flow/features/ConfirmationDialog";
import { useT } from "@flow/lib/i18n";

type Props = {
  showBeforeDeleteDialog: boolean;
  deferredDeleteRef: React.RefObject<{
    resolve: (val: boolean) => void;
  } | null>;
  onDialogClose: () => void;
};

const NodeDeletionDialog: React.FC<Props> = ({
  showBeforeDeleteDialog,
  deferredDeleteRef,
  onDialogClose,
}) => {
  const t = useT();

  const handleDialogSubmit = () => {
    if (deferredDeleteRef.current) {
      deferredDeleteRef.current.resolve(true);
      deferredDeleteRef.current = null;
    }
    onDialogClose();
  };

  const handleDialogCancel = () => {
    if (deferredDeleteRef.current) {
      deferredDeleteRef.current.resolve(false);
      deferredDeleteRef.current = null;
    }
    onDialogClose();
  };

  return (
    <ConfirmationDialog
      title={t("Are you sure you want to delete an input or output node?")}
      description={t(
        "Input and Output nodes are required for the workflow to run. By deleting a node you may cause unexpected behavior.",
      )}
      isOpen={!!showBeforeDeleteDialog}
      confirmDisabled={!showBeforeDeleteDialog}
      onClose={() => handleDialogCancel()}
      onConfirm={() => handleDialogSubmit()}
    />
  );
};

export default NodeDeletionDialog;
