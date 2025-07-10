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
        "This action cannot be undone. This will permanently delete your asset. Deletion of an asset could cause issues projects or deployments that are referencing this asset.",
      )}
      isOpen={!!assetToBeDeleted}
      confirmDisabled={!assetToBeDeleted}
      onClose={() => setAssetToBeDeleted(undefined)}
      onConfirm={() => assetToBeDeleted && onDeleteAsset(assetToBeDeleted)}
    />
  );
};

export { AssetDeletionDialog };
