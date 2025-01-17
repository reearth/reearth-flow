import ConfirmationDialog from "@flow/features/ConfirmationDialog";
import { useT } from "@flow/lib/i18n";
import { Trigger } from "@flow/types";

type Props = {
  triggerTobeDeleted: Trigger | undefined;
  setTriggerToBeDeleted: (trigger?: Trigger) => void;
  onTriggerDelete: (trigger?: Trigger) => Promise<void>;
};

const TriggerDeletionDialog: React.FC<Props> = ({
  triggerTobeDeleted,
  setTriggerToBeDeleted,
  onTriggerDelete,
}) => {
  const t = useT();

  return (
    <ConfirmationDialog
      title={t("Are you absolutely sure?")}
      description={t(
        "This action cannot be undone. This will permanently delete your trigger and remove your data from our servers.",
      )}
      isOpen={!!triggerTobeDeleted}
      confirmDisabled={!triggerTobeDeleted}
      onClose={() => setTriggerToBeDeleted(undefined)}
      onConfirm={async () => {
        if (triggerTobeDeleted) {
          await onTriggerDelete(triggerTobeDeleted);
        }
      }}
    />
  );
};

export { TriggerDeletionDialog };
