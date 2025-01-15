import ConfirmationDialog from "@flow/features/ConfirmationDialog";
import { useT } from "@flow/lib/i18n";
import { Deployment } from "@flow/types";

type Props = {
  deploymentToBeDeleted: Deployment | undefined;
  setDeploymentToBeDeleted: (deployment?: Deployment) => void;
  onDeploymentDelete: (deployment?: Deployment) => Promise<void>;
};

const DeploymentDeletionDialog: React.FC<Props> = ({
  deploymentToBeDeleted,
  setDeploymentToBeDeleted,
  onDeploymentDelete,
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
        deploymentToBeDeleted && onDeploymentDelete(deploymentToBeDeleted)
      }
    />
  );
};

export { DeploymentDeletionDialog };
