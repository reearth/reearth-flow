import { Input } from "@flow/components";
import { WorkflowVariable } from "@flow/types";

type Props = {
  id?: string;
  max?: number;
  min?: number;
  className?: string;
  variable: Pick<WorkflowVariable, "defaultValue">;
  onDefaultValueChange: (newValue: string) => void;
};

export const ColorDefaultValueInput: React.FC<Props> = ({
  id = "default-color-picker",
  className,
  variable,
  onDefaultValueChange,
}) => {
  return (
    <Input
      id={id}
      type="color"
      className={className}
      value={variable.defaultValue || "#000000"}
      onChange={(e) => onDefaultValueChange(e.target.value)}
    />
  );
};

export default ColorDefaultValueInput;
