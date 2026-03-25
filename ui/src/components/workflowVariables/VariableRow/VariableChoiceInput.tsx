import {
  Checkbox,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type VariableChoiceInputProps = {
  variable: any;
  index: number;
  choices: { value: string; label: string }[];
  onDefaultValueChange: (index: number, newValue: any) => void;
};

export default function VariableChoiceInput({
  index,
  choices,
  variable,
  onDefaultValueChange,
}: VariableChoiceInputProps) {
  const t = useT();

  if (variable.config.allowMultiple) {
    return (
      <div className="space-y-2">
        {choices.map((option: { value: string; label: string }) => {
          return (
            <div
              key={`checkbox-${option.value}-${index}`}
              className="flex items-center space-x-2">
              <Checkbox
                id={`default-option-${index}`}
                checked={
                  Array.isArray(variable.defaultValue) &&
                  variable.defaultValue.includes(option.value)
                }
                onCheckedChange={(checked) => {
                  let newValue = Array.isArray(variable.defaultValue)
                    ? [...variable.defaultValue]
                    : [];
                  if (checked) {
                    newValue.push(option.value);
                  } else {
                    newValue = newValue.filter((v) => v !== option.value);
                  }
                  onDefaultValueChange(index, newValue);
                }}
              />
              <Label htmlFor={`default-option-${index}`}>{option.label}</Label>
            </div>
          );
        })}
      </div>
    );
  }
  if (variable.config.displayMode === "radio") {
    return (
      <div className="space-y-2">
        {choices.map((option: { value: string; label: string }) => {
          return (
            <div
              key={`checkbox-${option.value}-${index}`}
              className="flex items-center space-x-2">
              <Checkbox
                id={`default-option-${index}`}
                checked={variable.defaultValue === option.value}
                onCheckedChange={(checked) =>
                  onDefaultValueChange(index, checked ? option.value : null)
                }
              />
              <Label htmlFor={`default-option-${index}`}>{option.label}</Label>
            </div>
          );
        })}
      </div>
    );
  } else {
    return (
      <Select
        value={variable.defaultValue}
        onValueChange={(newValue) => onDefaultValueChange(index, newValue)}>
        <SelectTrigger className="h-9 w-[150px]">
          <SelectValue placeholder={t("Select an option")} />
        </SelectTrigger>
        <SelectContent>
          {choices.map((option: { value: string; label: string }) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    );
  }
}
