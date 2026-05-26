import { AssetDefaultSelectionInput } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { AwarenessUser, WorkflowVariable } from "@flow/types";

type Props = {
  variable: WorkflowVariable;
  fieldFocusMap?: Record<string, AwarenessUser[]>;
  onUpdate: (variable: WorkflowVariable) => void;
  onDialogOpen: (dialog: "assets" | "cms") => void;
  onFieldFocus?: (field: string | null) => void;
};

export const DefaultEditor: React.FC<Props> = ({
  variable,
  fieldFocusMap,
  onUpdate,
  onDialogOpen,
  onFieldFocus,
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
        fieldFocusMap={fieldFocusMap}
        onDefaultValueChange={handleDefaultValueChange}
        onDialogOpen={onDialogOpen}
        onFieldFocus={onFieldFocus}
      />
      <p className="mt-1 text-sm text-muted-foreground">
        {t("The default value to use when this variable is not set.")}
      </p>
    </div>
  );
};
