import ConfirmationDialog from "@flow/features/ConfirmationDialog";
import { useT } from "@flow/lib/i18n";

type Props = {
  assetToBeDeleted: string | undefined;
  setAssetToBeDeleted: (asset?: string) => void;
  onDeleteAsset: (id: string) => void;
};

const AssetDeletionDialog: React.FC<Props> = ({
  assetToBeDeleted,
  setAssetToBeDeleted,
  onDeleteAsset,
}) => {
  const t = useT();
  return (
    <ConfirmationDialog
      title={t("Are you absolutely sure?")}
      description={t(
        "This action cannot be undone. Deleting this asset will permanently remove it from your account and may cause issues with any projects or deployments that reference it. Please ensure this asset is no longer in use before proceeding.",
      )}
      isOpen={!!assetToBeDeleted}
      confirmDisabled={!assetToBeDeleted}
      onClose={() => setAssetToBeDeleted(undefined)}
      onConfirm={() => assetToBeDeleted && onDeleteAsset(assetToBeDeleted)}
    />
  );
};

export { AssetDeletionDialog };
