import {
  Checkbox,
  Label,
  RadioGroup,
  RadioGroupItem,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { AnyWorkflowVariable } from "@flow/types";

type ChoiceConfig = {
  allowMultiple?: boolean;
  displayMode?: string;
};

type VariableChoiceInputProps = {
  variable: AnyWorkflowVariable;
  index: number;
  choices: { value: string; label: string }[];
  onDefaultValueChange: (
    index: number,
    newValue: string | string[] | null,
  ) => void;
};
export default function VariableChoiceInput({
  index,
  choices,
  variable,
  onDefaultValueChange,
}: VariableChoiceInputProps) {
  const t = useT();

  if (
    variable.config &&
    variable.type === "choice" &&
    (variable.config as ChoiceConfig).allowMultiple
  ) {
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
                  const isChecked = !!checked;
                  let newValue = Array.isArray(variable.defaultValue)
                    ? [...variable.defaultValue]
                    : [];
                  if (isChecked) {
                    newValue.push(option.value);
                  } else {
                    newValue = newValue.filter((v) => v !== option.value);
                  }
                  onDefaultValueChange(index, newValue);
                }}
              />
              <Label htmlFor={`default-option-${option.value}-${index}`}>
                {option.label}
              </Label>
            </div>
          );
        })}
      </div>
    );
  }
  if (
    variable.config &&
    variable.type === "choice" &&
    (variable.config as ChoiceConfig).displayMode === "radio"
  ) {
    return (
      <RadioGroup
        value={
          typeof variable.defaultValue === "string" ? variable.defaultValue : ""
        }
        onValueChange={(newValue) => onDefaultValueChange(index, newValue)}>
        {choices.map(
          (option: { value: string; label: string }, idx: number) => (
            <div
              key={`radio-${option.value}-${index}`}
              className="flex items-center space-x-2">
              <RadioGroupItem
                value={option.value}
                id={`option-${index}-${idx}`}
              />
              <Label htmlFor={`option-${index}-${idx}`}>{option.label}</Label>
            </div>
          ),
        )}
      </RadioGroup>
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
