import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useProjectVars } from "@flow/hooks";
import { VarType } from "@flow/types";

type VariableTypeSelectorProps = {
  value: VarType;
  onValueChange: (value: VarType) => void;
  disabled?: boolean;
};

export default function VariableTypeSelector({
  value,
  onValueChange,
  disabled = false,
}: VariableTypeSelectorProps) {
  const { userFacingName } = useProjectVars();
  return (
    <Select value={value} onValueChange={onValueChange} disabled={disabled}>
      <SelectTrigger className="w-full">
        <SelectValue placeholder="Select variable type" />
      </SelectTrigger>
      <SelectContent>
        {Object.keys(userFacingName).map((key) => (
          <SelectItem key={key} value={key}>
            <div className="flex flex-col">
              <span>{userFacingName[key as keyof typeof userFacingName]}</span>
            </div>
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
