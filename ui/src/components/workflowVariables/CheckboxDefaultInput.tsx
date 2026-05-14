import { Checkbox, Label, RadioGroup, RadioGroupItem } from "@flow/components";
import { useT } from "@flow/lib/i18n";

type ChoiceOption = { value: string; label: string };

type Props = {
  choices: ChoiceOption[];
  allowMultiple?: boolean;
  value?: string | string[];
  noDefaultOption?: boolean;
  onChange: (newValue: string | string[]) => void;
};

export const CheckboxDefaultInput: React.FC<Props> = ({
  choices,
  allowMultiple = false,
  noDefaultOption = false,
  value,
  onChange,
}) => {
  const t = useT();
  if (allowMultiple) {
    const selected = Array.isArray(value) ? value : [];
    return (
      <div className="space-y-2">
        {choices.map((option) => (
          <div key={option.value} className="flex items-center space-x-2">
            <Checkbox
              id={`choice-${option.value}`}
              checked={selected.includes(option.value)}
              onCheckedChange={(checked) => {
                const next = checked
                  ? [...selected, option.value]
                  : selected.filter((v) => v !== option.value);
                onChange(next);
              }}
            />
            <Label htmlFor={`choice-${option.value}`}>{option.label}</Label>
          </div>
        ))}
      </div>
    );
  }

  return (
    <RadioGroup
      value={typeof value === "string" ? value : ""}
      onValueChange={onChange}>
      {noDefaultOption && (
        <div className="flex items-center space-x-2">
          <RadioGroupItem value="" id="no-default" />
          <Label htmlFor="no-default" className="text-muted-foreground">
            {t("No default")}
          </Label>
        </div>
      )}
      {choices.map((option, idx) => (
        <div key={option.value} className="flex items-center space-x-2">
          <RadioGroupItem
            value={option.value}
            id={`radio-${option.value}-${idx}`}
          />
          <Label htmlFor={`radio-${option.value}-${idx}`}>{option.label}</Label>
        </div>
      ))}
    </RadioGroup>
  );
};

export default CheckboxDefaultInput;
