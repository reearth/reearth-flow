import { Input, Label } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { ProjectVariable, NumberConfig } from "@flow/types";

type Props = {
  variable: ProjectVariable;
  onUpdate: (variable: ProjectVariable) => void;
};

export const NumberEditor: React.FC<Props> = ({ variable, onUpdate }) => {
  const t = useT();

  // Get number config with defaults
  const config = (variable.config as NumberConfig) || {};

  const handleDefaultValueChange = (value: string) => {
    // Allow empty string for clearing the field
    if (value === "") {
      onUpdate({
        ...variable,
        defaultValue: "",
      });
      return;
    }

    // Only allow valid number formats (including decimals and negative numbers)
    const numberRegex = /^-?\d*\.?\d*$/;
    if (numberRegex.test(value)) {
      // Validate against min/max constraints
      const numValue = parseFloat(value);
      if (!isNaN(numValue)) {
        if (config.min !== undefined && numValue < config.min) {
          return; // Block input if below minimum
        }
        if (config.max !== undefined && numValue > config.max) {
          return; // Block input if above maximum
        }
      }

      onUpdate({
        ...variable,
        defaultValue: value,
      });
    }
    // If invalid number format, don't update (effectively blocks the input)
  };

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

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    // Allow: backspace, delete, tab, escape, enter
    if (
      [8, 9, 27, 13, 46].indexOf(e.keyCode) !== -1 ||
      // Allow: Ctrl+A, Ctrl+C, Ctrl+V, Ctrl+X
      (e.keyCode === 65 && e.ctrlKey === true) ||
      (e.keyCode === 67 && e.ctrlKey === true) ||
      (e.keyCode === 86 && e.ctrlKey === true) ||
      (e.keyCode === 88 && e.ctrlKey === true) ||
      // Allow: home, end, left, right
      (e.keyCode >= 35 && e.keyCode <= 39)
    ) {
      return;
    }

    // Ensure that it is a number or decimal point or minus sign
    if (
      (e.shiftKey || e.keyCode < 48 || e.keyCode > 57) &&
      (e.keyCode < 96 || e.keyCode > 105) &&
      e.keyCode !== 190 &&
      e.keyCode !== 110 && // decimal point
      e.keyCode !== 189 &&
      e.keyCode !== 109
    ) {
      // minus sign
      e.preventDefault();
    }
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
        <Input
          id="default-value"
          type="text"
          value={variable.defaultValue || ""}
          onChange={(e) => handleDefaultValueChange(e.target.value)}
          onKeyDown={handleKeyDown}
          onFocus={(e) => e.stopPropagation()}
          placeholder={t("Enter numeric value")}
          className="mt-1"
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
