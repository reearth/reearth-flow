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
      title={t("Are you sure you want to delete this asset?")}
      description={t(
        "This action cannot be undone. The asset will be permanently deleted. Warning: If this asset is being used in any projects or deployments, deleting it may cause those to malfunction.",
      )}
      isOpen={!!assetToBeDeleted}
      confirmDisabled={!assetToBeDeleted}
      onClose={() => setAssetToBeDeleted(undefined)}
      onConfirm={() => assetToBeDeleted && onDeleteAsset(assetToBeDeleted)}
    />
  );
};

export { AssetDeletionDialog };
