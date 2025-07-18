import { ChangeEvent, useCallback, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  Label,
  DialogFooter,
  Input,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Asset } from "@flow/types";

type Props = {
  assetToBeEdited: Asset;
  setAssetToBeEdited: (asset: Asset | undefined) => void;
  onUpdateAsset: (updatedName: string) => void;
};

const AssetEditDialog: React.FC<Props> = ({
  assetToBeEdited,
  setAssetToBeEdited,
  onUpdateAsset,
}) => {
  const t = useT();
  const [updatedName, setUpdatedName] = useState(assetToBeEdited?.name || "");
  const handleAssetNameChange = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => {
      setUpdatedName(e.target.value);
    },
    [],
  );
  return (
    <Dialog
      open={!!assetToBeEdited}
      onOpenChange={() => setAssetToBeEdited(undefined)}>
      <DialogContent size="sm">
        <DialogTitle>{t("Edit Asset")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex flex-col">
            <Label>{t("Name")}</Label>
            <Input
              value={updatedName}
              onChange={handleAssetNameChange}
              placeholder={t("Give your asset name")}
            />
          </DialogContentSection>
          <DialogContentSection>
            <p className="dark:font-light">
              {t("Are you sure you want to proceed?")}
            </p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            disabled={
              updatedName === assetToBeEdited.name || !updatedName.trim()
            }
            onClick={() => onUpdateAsset(updatedName)}>
            {t("Update Asset")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { AssetEditDialog };
