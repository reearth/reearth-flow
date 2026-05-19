import { Input } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { WorkflowVariable } from "@flow/types";

type Props = {
  id?: string;
  max?: number;
  min?: number;
  className?: string;
  variable: Pick<WorkflowVariable, "defaultValue" | "config">;
  onDefaultValueChange: (newValue: number | null) => void;
};

export const NumberDefaultValueInput: React.FC<Props> = ({
  id = "default-number",
  max,
  min,
  className,
  variable,
  onDefaultValueChange,
}) => {
  const t = useT();

  const handleValueChange = (value: string) => {
    // Allow empty string for clearing the field
    if (value === "") {
      onDefaultValueChange(null);
      return;
    }

    // Only allow valid number formats (including decimals and negative numbers)
    const numberRegex = /^-?\d*\.?\d*$/;
    if (numberRegex.test(value)) {
      // Validate against min/max constraints
      const numValue = parseFloat(value);

      // Only update if it's a valid complete number (not partial input like "1." or "-")
      if (!isNaN(numValue) && value !== "-" && !value.endsWith(".")) {
        if (
          (min !== undefined && numValue < min) ||
          (max !== undefined && numValue > max)
        ) {
          return; // Block input if below temp minimum or above temp maxiumum
        } else {
          // Type guard: check if config is NumberConfig by checking for min/max
          if (
            variable.config &&
            typeof (variable.config as any).min === "number"
          ) {
            const numConfig = variable.config as { min?: number; max?: number };
            if (
              (numConfig.min !== undefined && numValue < numConfig.min) ||
              (numConfig.max !== undefined && numValue > numConfig.max)
            ) {
              return; // Block input if below minimum or above maximum
            }
          }
        }

        onDefaultValueChange(numValue);
      }
    }
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

  return (
    <Input
      id={id}
      type="number"
      value={variable.defaultValue ?? ""}
      onChange={(e) => handleValueChange(e.target.value)}
      onKeyDown={handleKeyDown}
      onFocus={(e) => e.stopPropagation()}
      placeholder={t("Enter numeric value")}
      className={className}
    />
  );
};

export default NumberDefaultValueInput;
