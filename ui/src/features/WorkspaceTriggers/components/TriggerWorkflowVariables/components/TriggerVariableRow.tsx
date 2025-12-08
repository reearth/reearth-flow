import { Input, Switch, TextArea } from "@flow/components";

import { VariableMapping } from "..";

import TriggerVariableArrayInput from "./TriggerVariableArrayInput";

type Props = {
  variable: VariableMapping;
  index: number;
  onDefaultValueChange: (index: number, newValue: any) => void;
};

const TriggerVariableRow: React.FC<Props> = ({
  variable,
  index,
  onDefaultValueChange,
}) => {
  switch (variable.type) {
    case "array":
      return (
        <TriggerVariableArrayInput
          value={
            Array.isArray(variable.defaultValue) ? variable.defaultValue : []
          }
          onChange={(newValue) => onDefaultValueChange(index, newValue)}
        />
      );
    case "yes_no":
      return (
        <Switch
          checked={Boolean(variable.defaultValue)}
          onCheckedChange={(checked) => onDefaultValueChange(index, checked)}
        />
      );
    case "number":
      return (
        <Input
          id={`default-${index}`}
          type="number"
          value={variable.defaultValue}
          onChange={(e) => {
            const value = parseFloat(e.target.value);
            onDefaultValueChange(index, value);
          }}
        />
      );
    case "color":
      return (
        <div className="flex items-center gap-2">
          <Input
            id={`default-${index}`}
            className="h-6 w-6 rounded border p-0"
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
        <Input
          id={`default-${index}`}
          type={"datetime-local"}
          value={variable.defaultValue}
          onChange={(e) => {
            onDefaultValueChange(index, e.target.value);
          }}
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
  }
};

export { TriggerVariableRow };
