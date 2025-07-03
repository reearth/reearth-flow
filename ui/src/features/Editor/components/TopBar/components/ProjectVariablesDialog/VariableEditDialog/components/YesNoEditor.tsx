import { Label, Switch } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { ProjectVariable } from "@flow/types";

type Props = {
  variable: ProjectVariable;
  onUpdate: (variable: ProjectVariable) => void;
};

export const YesNoEditor: React.FC<Props> = ({ variable, onUpdate }) => {
  const t = useT();

  const handleDefaultValueChange = (checked: boolean) => {
    onUpdate({
      ...variable,
      defaultValue: checked,
    });
  };

  // Convert the default value to boolean
  const getBooleanValue = (value: any): boolean => {
    if (typeof value === "boolean") return value;
    if (typeof value === "string") {
      const lowerValue = value.toLowerCase();
      return lowerValue === "true" || lowerValue === "yes" || lowerValue === "1";
    }
    if (typeof value === "number") return value !== 0;
    return false;
  };

  const isChecked = getBooleanValue(variable.defaultValue);

  return (
    <div className="space-y-4">
      <div>
        <Label className="text-sm font-medium">
          {t("Default Value")}
        </Label>
        <div className="mt-2 flex items-center space-x-3">
          <Switch
            checked={isChecked}
            onCheckedChange={handleDefaultValueChange}
          />
          <span className="text-sm text-muted-foreground">
            {isChecked ? t("Yes") : t("No")}
          </span>
        </div>
        <p className="mt-2 text-sm text-muted-foreground">
          {t("The default yes/no value to use when this variable is not set.")}
        </p>
      </div>
    </div>
  );
};