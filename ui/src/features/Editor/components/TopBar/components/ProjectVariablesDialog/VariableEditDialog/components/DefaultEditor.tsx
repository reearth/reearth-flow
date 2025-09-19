import { ArchiveIcon, DatabaseIcon } from "@phosphor-icons/react";

import { Button, Input, Label } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { ProjectVariable } from "@flow/types";

type Props = {
  variable: ProjectVariable;
  onUpdate: (variable: ProjectVariable) => void;
  onDialogOpen: (dialog: "assets" | "cms") => void;
};

export const DefaultEditor: React.FC<Props> = ({
  variable,
  onUpdate,
  onDialogOpen,
}) => {
  const t = useT();

  const handleDefaultValueChange = (value: string) => {
    onUpdate({
      ...variable,
      defaultValue: value,
    });
  };

  return (
    <div className="space-y-4">
      <div>
        <div className="mb-1 flex items-center justify-between pb-1">
          <Label htmlFor="default-value" className="text-sm font-medium">
            {t("Default Value")}
          </Label>
          <div className="flex gap-2">
            <Button
              onClick={() => onDialogOpen("assets")}
              variant="outline"
              size="sm">
              <ArchiveIcon className="h-4 w-4" />
              {t("Asset")}
            </Button>
            <Button
              onClick={() => onDialogOpen("cms")}
              variant="outline"
              size="sm">
              <DatabaseIcon className="h-4 w-4" />
              {t("CMS")}
            </Button>
          </div>
        </div>
        <Input
          id="default-value"
          value={variable.defaultValue || ""}
          onChange={(e) => handleDefaultValueChange(e.target.value)}
          placeholder={t("Enter default value")}
          className="mt-1"
        />
        <p className="mt-1 text-sm text-muted-foreground">
          {t("The default value to use when this variable is not set.")}
        </p>
      </div>
    </div>
  );
};
