import { Input } from "@flow/components";

type TriggerVariableArrayInputProps = {
  value: any[];
  onChange: (newValue: any[]) => void;
  className?: string;
};

export default function TriggerVariableArrayInput({
  value,
  onChange,
  className,
}: TriggerVariableArrayInputProps) {
  const handleUpdateItem = (index: number, newValue: string) => {
    const updatedArray = [...value];
    updatedArray[index] = newValue;
    onChange(updatedArray);
  };

  return (
    <div className={className}>
      <div className="space-y-2">
        {value.map((item, index) => (
          <div key={index} className="flex items-center">
            <span className="w-6 text-sm text-muted-foreground">
              {index + 1}.
            </span>
            <Input
              value={String(item)}
              onChange={(e) => handleUpdateItem(index, e.target.value)}
              className="flex-1"
            />
          </div>
        ))}
      </div>
    </div>
  );
}
