import { FileIcon } from "@phosphor-icons/react";

import { Button, CmsLogo, Input, Label } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { WorkflowVariable } from "@flow/types";

type Props = {
  id?: string;
  variable: Pick<WorkflowVariable, "defaultValue" | "config">;
  onDialogOpen: (dialog: "assets" | "cms") => void;
  onDefaultValueChange: (newValue: string) => void;
};

export const AssetDefaultSelectionInput: React.FC<Props> = ({
  id = "default-asset-selection",
  variable,
  onDialogOpen,
  onDefaultValueChange,
}) => {
  const t = useT();
  return (
    <div id={id}>
      <div className="mb-1 flex items-center justify-between pb-1">
        <Label htmlFor="default-value" className="text-sm font-medium">
          {t("Default Value")}
        </Label>
        <div className="flex gap-2">
          <Button
            onClick={() => onDialogOpen("assets")}
            variant="outline"
            size="sm">
            <FileIcon className="h-4 w-4" />
            {t("Workspace Assets")}
          </Button>
          <Button
            onClick={() => onDialogOpen("cms")}
            variant="outline"
            size="sm">
            <CmsLogo className="h-4 w-4 text-white" />
            {t("CMS Integration")}
          </Button>
        </div>
      </div>
      <Input
        id="default-value"
        value={variable.defaultValue ?? ""}
        onChange={(e) => onDefaultValueChange(e.target.value)}
        placeholder={t("Enter default value")}
        className="mt-1"
      />
    </div>
  );
};

export default AssetDefaultSelectionInput;
