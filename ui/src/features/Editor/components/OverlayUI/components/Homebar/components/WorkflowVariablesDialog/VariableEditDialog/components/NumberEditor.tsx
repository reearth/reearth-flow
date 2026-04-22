import { Input, Label, NumberDefaultValueInput } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { WorkflowVariable, NumberConfig } from "@flow/types";

type Props = {
  variable: WorkflowVariable;
  onUpdate: (variable: WorkflowVariable) => void;
};

export const NumberEditor: React.FC<Props> = ({ variable, onUpdate }) => {
  const t = useT();

  // Get number config with defaults
  const config = (variable.config as NumberConfig) || {};

  const handleConfigChange = (
    configKey: keyof NumberConfig,
    value: number | undefined,
  ) => {
    const newConfig = {
      ...config,
      [configKey]: value,
    };

    onUpdate({
      ...variable,
      config: newConfig,
    });
  };

  // Helper to format constraint info
  const getConstraintText = () => {
    const constraints = [];
    if (config.min !== undefined) constraints.push(`min: ${config.min}`);
    if (config.max !== undefined) constraints.push(`max: ${config.max}`);
    return constraints.length > 0 ? ` (${constraints.join(", ")})` : "";
  };

  return (
    <div className="space-y-4">
      <div>
        <Label htmlFor="default-value" className="text-sm font-medium">
          {t("Default Value")}
        </Label>
        <NumberDefaultValueInput
          id="default-value"
          className="mt-1"
          variable={variable}
          onDefaultValueChange={(newValue) =>
            onUpdate({
              ...variable,
              defaultValue: newValue,
            })
          }
        />
        <p className="mt-1 text-sm text-muted-foreground">
          {t("The default numeric value to use when this variable is not set")}
          {getConstraintText()}.
        </p>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div>
          <Label htmlFor="min-value" className="text-sm font-medium">
            {t("Minimum Value")}
          </Label>
          <Input
            id="min-value"
            type="number"
            value={config.min ?? ""}
            onChange={(e) => {
              const value =
                e.target.value === "" ? undefined : parseFloat(e.target.value);
              handleConfigChange("min", value);
            }}
            onFocus={(e) => e.stopPropagation()}
            placeholder={t("No minimum")}
            className="mt-1"
          />
        </div>

        <div>
          <Label htmlFor="max-value" className="text-sm font-medium">
            {t("Maximum Value")}
          </Label>
          <Input
            id="max-value"
            type="number"
            value={config.max ?? ""}
            onChange={(e) => {
              const value =
                e.target.value === "" ? undefined : parseFloat(e.target.value);
              handleConfigChange("max", value);
            }}
            onFocus={(e) => e.stopPropagation()}
            placeholder={t("No maximum")}
            className="mt-1"
          />
        </div>
      </div>

      <p className="text-sm text-muted-foreground">
        {t(
          "Set minimum and maximum constraints for this number variable. Leave empty for no constraint.",
        )}
      </p>
    </div>
  );
};
