import { ColorDefaultValueInput, Input, Label } from "@flow/components";
import { paramsAwarenessStyles } from "@flow/components/SchemaForm/utils/awarenessTemplateStyles";
import { useT } from "@flow/lib/i18n";
import { AwarenessUser, WorkflowVariable } from "@flow/types";

type Props = {
  variable: WorkflowVariable;
  fieldFocusMap?: Record<string, AwarenessUser[]>;
  onUpdate: (variable: WorkflowVariable) => void;
  onFieldFocus?: (field: string | null) => void;
};

export const ColorEditor: React.FC<Props> = ({
  variable,
  fieldFocusMap,
  onUpdate,
  onFieldFocus,
}) => {
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
        <div className="mt-1 flex items-center gap-3 rounded">
          <ColorDefaultValueInput
            id="color-picker"
            className="h-10 w-16 rounded border p-1"
            variable={variable}
            onDefaultValueChange={(newValue) => handleColorChange(newValue)}
          />
          <Input
            value={variable.defaultValue || ""}
            onChange={(e) => handleColorChange(e.target.value)}
            placeholder={t("Enter color hex code")}
            className="flex-1"
            style={paramsAwarenessStyles(fieldFocusMap?.["defaultValue"])}
            onFocus={() => onFieldFocus?.("defaultValue")}
            onBlur={() => onFieldFocus?.(null)}
          />
        </div>
        <p className="mt-1 text-sm text-muted-foreground">
          {t("The default color value to use when this variable is not set.")}
        </p>
      </div>
    </div>
  );
};
