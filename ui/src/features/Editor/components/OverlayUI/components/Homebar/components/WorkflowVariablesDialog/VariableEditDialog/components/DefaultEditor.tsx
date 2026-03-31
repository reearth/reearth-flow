import { AssetDefaultSelectionInput } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { WorkflowVariable } from "@flow/types";

type Props = {
  variable: WorkflowVariable;
  onUpdate: (variable: WorkflowVariable) => void;
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
      defaultValue: value === "" ? null : value,
    });
  };

  return (
    <div className="space-y-4">
      <AssetDefaultSelectionInput
        variable={variable}
        onDefaultValueChange={handleDefaultValueChange}
        onDialogOpen={onDialogOpen}
      />
      <p className="mt-1 text-sm text-muted-foreground">
        {t("The default value to use when this variable is not set.")}
      </p>
    </div>
  );
};
