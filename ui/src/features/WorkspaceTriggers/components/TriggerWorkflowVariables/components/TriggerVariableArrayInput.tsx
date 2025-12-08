import { Input, Switch } from "@flow/components";

type TriggerVariableArrayInputProps = {
  value: any[];
  onChange: (newValue: any[]) => void;
};

export default function TriggerVariableArrayInput({
  value,
  onChange,
}: TriggerVariableArrayInputProps) {
  const handleUpdateItem = (index: number, newValue: any) => {
    const updatedArray = [...value];
    updatedArray[index] = newValue;
    onChange(updatedArray);
  };
  return (
    <div className="space-y-2">
      {value.map((item, index) => (
        <div key={index} className="flex items-center">
          <span className="w-6 text-sm text-muted-foreground">
            {index + 1}.
          </span>
          {typeof item === "boolean" ? (
            <div
              id={`default-${index}`}
              className="flex items-center space-x-3">
              <span className="text-sm font-medium">
                {item ? "true" : "false"}
              </span>
              <Switch
                checked={Boolean(item)}
                onCheckedChange={(checked) => handleUpdateItem(index, checked)}
              />
            </div>
          ) : (
            <Input
              id={`default-${index}`}
              type={typeof item === "number" ? "number" : "text"}
              value={item}
              onChange={(e) => {
                const value = parseFloat(e.target.value);
                handleUpdateItem(index, value);
              }}
            />
          )}
        </div>
      ))}
    </div>
  );
}
