import { Input, Label } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { ProjectVariable } from "@flow/types";

type Props = {
  variable: ProjectVariable;
  onUpdate: (variable: ProjectVariable) => void;
};

export const ColorEditor: React.FC<Props> = ({ variable, onUpdate }) => {
  const t = useT();

  const handleColorChange = (value: string) => {
    onUpdate({
      ...variable,
      defaultValue: value,
    });
  };

  return (
    <div className="space-y-4">
      <div>
        <Label htmlFor="color-picker" className="text-sm font-medium">
          {t("Default Color")}
        </Label>
        <div className="mt-1 flex items-center gap-3">
          <Input
            id="color-picker"
            type="color"
            value={variable.defaultValue || "#000000"}
            onChange={(e) => handleColorChange(e.target.value)}
            className="h-10 w-16 rounded border p-1"
          />
          <Input
            value={variable.defaultValue || ""}
            onChange={(e) => handleColorChange(e.target.value)}
            placeholder={t("Enter color hex code")}
            className="flex-1"
          />
        </div>
        <p className="mt-1 text-sm text-muted-foreground">
          {t("The default color value to use when this variable is not set.")}
        </p>
      </div>
    </div>
  );
};