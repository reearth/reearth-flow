import { PlusIcon, TrashIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { Input, Button, IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";

type SimpleArrayInputProps = {
  value: any[];
  onChange: (newValue: any[]) => void;
  className?: string;
};

export default function SimpleArrayInput({
  value,
  onChange,
  className,
}: SimpleArrayInputProps) {
  const t = useT();
  const [newItem, setNewItem] = useState("");

  const handleAddItem = () => {
    if (newItem.trim()) {
      onChange([...value, newItem.trim()]);
      setNewItem("");
    }
  };

  const handleRemoveItem = (index: number) => {
    onChange(value.filter((_, i) => i !== index));
  };

  const handleUpdateItem = (index: number, newValue: string) => {
    const updatedArray = [...value];
    updatedArray[index] = newValue;
    onChange(updatedArray);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleAddItem();
    }
  };

  return (
    <div className={className}>
      {/* Existing items */}
      <div className="space-y-2">
        {value.map((item, index) => (
          <div key={index} className="flex items-center gap-2">
            <span className="w-6 text-sm text-muted-foreground">
              {index + 1}.
            </span>
            <Input
              value={String(item)}
              onChange={(e) => handleUpdateItem(index, e.target.value)}
              className="flex-1"
            />
            <IconButton
              icon={<TrashIcon size={14} />}
              size="sm"
              variant="ghost"
              onClick={() => handleRemoveItem(index)}
              tooltipText={t("Remove item")}
            />
          </div>
        ))}
      </div>

      {/* Add new item */}
      <div className="mt-2 flex items-center gap-2">
        <Input
          value={newItem}
          onChange={(e) => setNewItem(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={t("Add new item...")}
          className="flex-1"
        />
        <Button
          size="sm"
          onClick={handleAddItem}
          disabled={!newItem.trim()}
          className="flex items-center gap-1">
          <PlusIcon size={14} />
          {t("Add")}
        </Button>
      </div>

      {value.length === 0 && (
        <div className="rounded-md border-2 border-dashed border-muted-foreground/25 p-4 text-center">
          <p className="text-sm text-muted-foreground">
            {t("No items yet. Add items above.")}
          </p>
        </div>
      )}
    </div>
  );
}
