import {
  CheckboxDefaultInput,
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

  if (variable.config && variable.type === "choice") {
    const config = variable.config as ChoiceConfig;
    if (config.allowMultiple || config.displayMode === "radio") {
      return (
        <CheckboxDefaultInput
          choices={choices}
          allowMultiple={!!config.allowMultiple}
          value={variable.defaultValue}
          onChange={(v) => onDefaultValueChange(index, v)}
        />
      );
    }
  }

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
