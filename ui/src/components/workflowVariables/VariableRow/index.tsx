import { useCallback } from "react";

import {
  DateTimeDefaultValueInput,
  Input,
  Switch,
  TextArea,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { AnyWorkflowVariable, TriggerVariableConfig } from "@flow/types";

import VariableArrayInput from "./VariableArrayInput";
import VariableChoiceInput from "./VariableChoiceInput";

type Props = {
  variable: TriggerVariableConfig | AnyWorkflowVariable;
  index: number;
  onDefaultValueChange: (index: number, newValue: any) => void;
};

const VariableRow: React.FC<Props> = ({
  variable,
  index,
  onDefaultValueChange,
}) => {
  const t = useT();

  const validateNumber = useCallback(
    (value: number) => {
      if (
        "config" in variable &&
        variable.config &&
        ("max" in variable.config || "min" in variable.config)
      ) {
        if (
          (variable.config.max !== undefined && value > variable.config.max) ||
          (variable.config.min !== undefined && value < variable.config.min)
        ) {
          return false;
        }
      }

      return true;
    },
    [variable],
  );

  switch (variable.type) {
    case "array":
      return (
        <VariableArrayInput
          value={
            Array.isArray(variable.defaultValue) ? variable.defaultValue : []
          }
          onChange={(newValue) => onDefaultValueChange(index, newValue)}
        />
      );
    case "yes_no":
      return (
        <div className="flex items-center space-x-3">
          <span className="text-sm font-medium">
            {variable.defaultValue ? t("Yes") : t("No")}
          </span>
          <Switch
            checked={Boolean(variable.defaultValue)}
            onCheckedChange={(checked) => onDefaultValueChange(index, checked)}
          />
        </div>
      );
    case "number":
      return (
        <Input
          id={`default-${index}`}
          type="number"
          value={variable.defaultValue}
          onChange={(e) => {
            const newValue = parseFloat(e.target.value);
            const validNumber = validateNumber(newValue);
            if (validNumber) {
              onDefaultValueChange(index, newValue);
            }
          }}
        />
      );
    case "choice":
      if (
        "config" in variable &&
        variable.config &&
        "choices" in variable.config
      ) {
        const rawChoices = variable.config.choices;
        const choices = rawChoices.map((choice: any) => {
          if (typeof choice === "string") {
            return { value: choice, label: choice };
          }
          return choice;
        });
        return (
          <VariableChoiceInput
            index={index}
            variable={variable}
            choices={choices}
            onDefaultValueChange={onDefaultValueChange}
          />
        );
      }
      return (
        <Input
          id={`default-${index}`}
          type="text"
          value={variable.defaultValue}
          onChange={(e) => {
            onDefaultValueChange(index, e.target.value);
          }}
        />
      );
    case "color":
      return (
        <div className="flex items-center gap-2">
          <Input
            id={`default-${index}`}
            className="h-6 w-6 rounded border p-0 hover:cursor-pointer"
            type={"color"}
            value={variable.defaultValue}
            onChange={(e) => {
              onDefaultValueChange(index, e.target.value);
            }}
          />
          <span className="font-mono text-sm">{variable.defaultValue}</span>
        </div>
      );
    case "datetime":
      return (
        <DateTimeDefaultValueInput
          id={`default-${index}`}
          variable={variable}
          onDefaultValueChange={(newValue) =>
            onDefaultValueChange(index, newValue)
          }
        />
      );
    case "text":
      if (
        typeof variable.defaultValue === "string" &&
        variable.defaultValue.length > 50
      ) {
        return (
          <TextArea
            id={`default-${index}`}
            value={variable.defaultValue}
            onChange={(e) => {
              onDefaultValueChange(index, e.target.value);
            }}
            className="min-h-[60px]"
          />
        );
      } else {
        return (
          <Input
            id={`default-${index}`}
            type="text"
            value={variable.defaultValue}
            onChange={(e) => {
              onDefaultValueChange(index, e.target.value);
            }}
          />
        );
      }
    default:
      console.error(
        `Unsupported variable type '${variable.type}' in Variable Row (index: ${index}).`,
      );
      return (
        <div className="text-sm font-semibold text-red-600">
          {t("Unsupported variable type")}:{" "}
          <span className="font-mono">{variable.type}</span>
        </div>
      );
  }
};

export { VariableRow };
