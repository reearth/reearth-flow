import { Input, Label } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { ProjectVariable } from "@flow/types";

type Props = {
  variable: ProjectVariable;
  onUpdate: (variable: ProjectVariable) => void;
};

export const DefaultEditor: React.FC<Props> = ({ variable, onUpdate }) => {
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
        <Label htmlFor="default-value" className="text-sm font-medium">
          {t("Default Value")}
        </Label>
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