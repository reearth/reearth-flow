import { Input, Label } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { WorkflowVariable } from "@flow/types";

type Props = {
  variable: WorkflowVariable;
  onUpdate: (variable: WorkflowVariable) => void;
};

export const AttributeNameEditor: React.FC<Props> = ({
  variable,
  onUpdate,
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
        <Label htmlFor="default-value" className="text-sm font-medium">
          {t("Default Attribute Name")}
        </Label>
        <Input
          id="default-value"
          value={variable.defaultValue || ""}
          onChange={(e) => handleDefaultValueChange(e.target.value)}
          placeholder={t("Enter default attribute name")}
          className="mt-1"
        />
        <p className="mt-1 text-sm text-muted-foreground">
          {t(
            "The default attribute name to use when this variable is not set.",
          )}
        </p>
      </div>
    </div>
  );
};
