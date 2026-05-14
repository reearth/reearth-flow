import { Input, Switch } from "@flow/components";

type Props = {
  value: any;
  itemType?: "string" | "number" | "boolean";
  onChange: (value: any) => void;
};

export const ArrayDefaultItemInput: React.FC<Props> = ({
  value,
  itemType,
  onChange,
}) => {
  const resolvedType =
    itemType ?? (typeof value as "string" | "number" | "boolean");

  switch (resolvedType) {
    case "boolean":
      return (
        <div className="flex items-center space-x-2">
          <Switch checked={!!value} onCheckedChange={onChange} />
          <span className="text-sm">{value ? "true" : "false"}</span>
        </div>
      );
    case "number":
      return (
        <Input
          type="number"
          value={value}
          onChange={(e) => onChange(parseFloat(e.target.value))}
          className="flex-1"
        />
      );
    case "string":
    default:
      return (
        <Input
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="flex-1"
        />
      );
  }
};

export default ArrayDefaultItemInput;
