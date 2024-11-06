import ConfirmationDialog from "@flow/features/ConfirmationDialog";
import { useT } from "@flow/lib/i18n";

type Props = {
  deploymentToBeDeleted: string | undefined;
  setDeploymentToBeDeleted: (deployment?: string) => void;
  onDeleteDeployment: (id: string) => void;
};

const DeploymentDeletionDialog: React.FC<Props> = ({
  deploymentToBeDeleted,
  setDeploymentToBeDeleted,
  onDeleteDeployment,
}) => {
  const t = useT();
  return (
    <ConfirmationDialog
      title={t("Are you absolutely sure?")}
      description={t(
        "This action cannot be undone. This will permanently delete your deployment and remove your data from our servers.",
      )}
      isOpen={!!deploymentToBeDeleted}
      confirmDisabled={!deploymentToBeDeleted}
      onClose={() => setDeploymentToBeDeleted(undefined)}
      onConfirm={() =>
        deploymentToBeDeleted && onDeleteDeployment(deploymentToBeDeleted)
      }
    />
  );
};

export { DeploymentDeletionDialog };
