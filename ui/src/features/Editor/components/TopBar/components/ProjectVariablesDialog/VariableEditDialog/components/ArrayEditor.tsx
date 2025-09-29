import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
  useSortable,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import {
  PlusIcon,
  DotsSixIcon,
  TrashIcon,
  ArchiveIcon,
  DatabaseIcon,
} from "@phosphor-icons/react";
import { useState, useEffect, useCallback } from "react";

import {
  Input,
  IconButton,
  Label,
  Switch,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Checkbox,
  Button,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { AnyProjectVariable, ArrayConfig } from "@flow/types";

type Props = {
  variable: AnyProjectVariable;
  assetUrl?: string | null;
  onUpdate: (variable: AnyProjectVariable) => void;
  onDialogOpen: (dialog: "assets" | "cms") => void;
  clearUrl: () => void;
};

export const ArrayEditor: React.FC<Props> = ({
  variable,
  assetUrl,
  onUpdate,
  onDialogOpen,
  clearUrl,
}) => {
  const t = useT();
  // Get the current array configuration from the config field
  const getArrayConfig = useCallback((): ArrayConfig => {
    if (variable.type === "array" && variable.config) {
      return variable.config as ArrayConfig;
    }

    // Default fallback
    return {
      itemType: "string",
      minItems: 0,
      maxItems: 10,
      allowDuplicates: true,
    };
  }, [variable.config, variable.type]);

  const [arrayConfig, setArrayConfig] = useState<ArrayConfig>(getArrayConfig());
  const [newItemText, setNewItemText] = useState("");

  // Get current array items
  const getArrayItems = (): any[] => {
    return Array.isArray(variable.defaultValue) ? variable.defaultValue : [];
  };

  const [arrayItems, setArrayItems] = useState<any[]>(getArrayItems());

  // Sync config and items when variable changes
  useEffect(() => {
    const newConfig = getArrayConfig();
    const newItems = Array.isArray(variable.defaultValue)
      ? variable.defaultValue
      : [];
    setArrayConfig(newConfig);
    setArrayItems(newItems);
  }, [getArrayConfig, variable.defaultValue]);

  useEffect(() => {
    if (assetUrl) {
      setNewItemText(assetUrl);
      clearUrl();
    }
  }, [assetUrl, clearUrl]);

  const updateVariable = (config: ArrayConfig, items?: any[]) => {
    setArrayConfig(config);

    const updatedVariable: AnyProjectVariable = {
      ...variable,
      config: variable.type === "array" ? config : variable.config,
      defaultValue: items !== undefined ? items : arrayItems,
    };

    onUpdate(updatedVariable);
  };

  const convertValue = (value: string, itemType: string): any => {
    switch (itemType) {
      case "number": {
        const num = parseFloat(value);
        return isNaN(num) ? 0 : num;
      }
      case "boolean":
        return value.toLowerCase() === "true" || value === "1";
      case "string":
      default:
        return value;
    }
  };

  const handleAddItem = () => {
    if (!newItemText.trim()) return;

    const convertedValue = convertValue(
      newItemText.trim(),
      arrayConfig.itemType || "string",
    );

    // Check for duplicates if not allowed
    if (!arrayConfig.allowDuplicates && arrayItems.includes(convertedValue)) {
      return;
    }

    // Check max items limit
    if (arrayConfig.maxItems && arrayItems.length >= arrayConfig.maxItems) {
      return;
    }

    const newItems = [...arrayItems, convertedValue];
    setArrayItems(newItems);
    updateVariable(arrayConfig, newItems);
    setNewItemText("");
  };

  const handleRemoveItem = (index: number) => {
    const newItems = arrayItems.filter((_, i) => i !== index);
    setArrayItems(newItems);
    updateVariable(arrayConfig, newItems);
  };

  const handleUpdateItem = (index: number, value: string) => {
    const convertedValue = convertValue(
      value,
      arrayConfig.itemType || "string",
    );
    const newItems = [...arrayItems];
    newItems[index] = convertedValue;
    setArrayItems(newItems);
    updateVariable(arrayConfig, newItems);
  };

  // Set up drag and drop sensors
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  );

  // Handle drag end for reordering items
  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (active.id !== over?.id) {
      const oldIndex = arrayItems.findIndex(
        (_, index) => `item-${index}` === active.id,
      );
      const newIndex = arrayItems.findIndex(
        (_, index) => `item-${index}` === over?.id,
      );

      if (oldIndex !== -1 && newIndex !== -1) {
        const newItems = [...arrayItems];
        const [movedItem] = newItems.splice(oldIndex, 1);
        newItems.splice(newIndex, 0, movedItem);

        setArrayItems(newItems);
        updateVariable(arrayConfig, newItems);
      }
    }
  };

  const handleConfigChange = (configUpdates: Partial<ArrayConfig>) => {
    const newConfig = { ...arrayConfig, ...configUpdates };
    updateVariable(newConfig);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleAddItem();
    }
  };

  const renderItemInput = (item: any, index: number) => {
    const stringValue = String(item);

    switch (arrayConfig.itemType) {
      case "boolean":
        return (
          <div className="flex items-center space-x-2">
            <Checkbox
              checked={!!item}
              onCheckedChange={(checked) =>
                handleUpdateItem(index, checked ? "true" : "false")
              }
            />
            <span>{item ? "true" : "false"}</span>
          </div>
        );
      case "number":
        return (
          <Input
            type="number"
            value={stringValue}
            onChange={(e) => handleUpdateItem(index, e.target.value)}
            className="flex-1"
          />
        );
      case "string":
      default:
        return (
          <Input
            value={stringValue}
            onChange={(e) => handleUpdateItem(index, e.target.value)}
            className="flex-1"
          />
        );
    }
  };

  return (
    <div className="space-y-6">
      {/* Configuration Options */}
      <div>
        <h3 className="mb-4 text-lg font-medium">{t("Configuration")}</h3>

        <div className="mb-6 grid grid-cols-2 gap-4">
          <div>
            <Label className="mb-2 block text-sm font-medium">
              {t("Item Type")}
            </Label>
            <Select
              value={arrayConfig.itemType || "string"}
              onValueChange={(value: "string" | "number" | "boolean") =>
                handleConfigChange({ itemType: value })
              }>
              <SelectTrigger>
                <SelectValue placeholder={t("Select item type")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="string">{t("Text")}</SelectItem>
                <SelectItem value="number">{t("Number")}</SelectItem>
                <SelectItem value="boolean">{t("True/False")}</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div>
            <Label className="mb-2 block text-sm font-medium">
              {t("Allow Duplicates")}
            </Label>
            <div className="mt-2 flex items-center space-x-2">
              <Switch
                checked={arrayConfig.allowDuplicates || false}
                onCheckedChange={(checked) =>
                  handleConfigChange({ allowDuplicates: checked })
                }
              />
              <span className="text-sm text-muted-foreground">
                {arrayConfig.allowDuplicates
                  ? t("Duplicate values allowed")
                  : t("Unique values only")}
              </span>
            </div>
          </div>

          <div>
            <Label className="mb-2 block text-sm font-medium">
              {t("Minimum Items")}
            </Label>
            <Input
              type="number"
              min="0"
              value={arrayConfig.minItems || 0}
              onChange={(e) =>
                handleConfigChange({ minItems: parseInt(e.target.value) || 0 })
              }
            />
          </div>

          <div>
            <Label className="mb-2 block text-sm font-medium">
              {t("Maximum Items")}
            </Label>
            <Input
              type="number"
              min="1"
              value={arrayConfig.maxItems || 10}
              onChange={(e) =>
                handleConfigChange({ maxItems: parseInt(e.target.value) || 10 })
              }
            />
          </div>
        </div>
      </div>

      {/* Array Items */}
      <div>
        <h3 className="mb-4 text-lg font-medium">
          {t("Array Items")} ({arrayItems.length})
        </h3>

        {/* Add new item */}
        <div className="mb-4 flex gap-2">
          {arrayConfig.itemType === "boolean" ? (
            <div className="flex flex-1 items-center space-x-2">
              <Checkbox
                checked={newItemText === "true"}
                onCheckedChange={(checked) =>
                  setNewItemText(checked ? "true" : "false")
                }
              />
              <span>{newItemText === "true" ? "true" : "false"}</span>
            </div>
          ) : (
            <Input
              type={arrayConfig.itemType === "number" ? "number" : "text"}
              value={newItemText}
              onChange={(e) => setNewItemText(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={t(
                `Enter new ${arrayConfig.itemType || "text"} value`,
              )}
              className="flex-1"
            />
          )}
          {arrayConfig.itemType === "string" && (
            <div className="flex gap-2">
              <Button
                onClick={() => onDialogOpen("assets")}
                variant="outline"
                size="sm">
                <ArchiveIcon className="h-4 w-4" />
                {t("Asset")}
              </Button>
              <Button
                onClick={() => onDialogOpen("cms")}
                variant="outline"
                size="sm">
                <DatabaseIcon className="h-4 w-4" />
                {t("CMS")}
              </Button>
            </div>
          )}
          <IconButton
            icon={<PlusIcon />}
            onClick={handleAddItem}
            disabled={
              !newItemText.trim() ||
              (!arrayConfig.allowDuplicates &&
                arrayItems.includes(
                  convertValue(
                    newItemText.trim(),
                    arrayConfig.itemType || "string",
                  ),
                )) ||
              (!!arrayConfig.maxItems &&
                arrayItems.length >= arrayConfig.maxItems)
            }
            tooltipText={t("Add item")}
          />
        </div>

        {/* Items list */}
        {arrayItems.length > 0 && (
          <div className="space-y-2">
            <DndContext
              sensors={sensors}
              collisionDetection={closestCenter}
              onDragEnd={handleDragEnd}>
              <SortableContext
                items={arrayItems.map((_, index) => `item-${index}`)}
                strategy={verticalListSortingStrategy}>
                {arrayItems.map((item, index) => (
                  <SortableArrayItem
                    key={`item-${index}`}
                    item={item}
                    index={index}
                    onUpdate={(value) => handleUpdateItem(index, value)}
                    onRemove={() => handleRemoveItem(index)}
                    renderInput={() => renderItemInput(item, index)}
                  />
                ))}
              </SortableContext>
            </DndContext>
          </div>
        )}

        {arrayItems.length === 0 && (
          <div className="rounded-md border-2 border-dashed border-muted-foreground/25 p-6 text-center">
            <p className="text-sm text-muted-foreground">
              {t("No items yet. Add some items to get started.")}
            </p>
          </div>
        )}
      </div>
    </div>
  );
};

// Sortable array item component
const SortableArrayItem: React.FC<{
  item: any;
  index: number;
  onUpdate: (value: string) => void;
  onRemove: () => void;
  renderInput: () => React.ReactNode;
}> = ({ index, onRemove, renderInput }) => {
  const t = useT();

  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: `item-${index}`,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="flex items-center gap-2 rounded-md border p-2"
      {...attributes}>
      <div
        className="flex cursor-grab touch-none items-center justify-center p-1 active:cursor-grabbing"
        {...listeners}>
        <DotsSixIcon size={16} className="text-muted-foreground" />
      </div>

      <span className="w-8 text-sm text-muted-foreground">{index + 1}.</span>

      {renderInput()}

      <IconButton
        icon={<TrashIcon size={14} />}
        size="sm"
        variant="ghost"
        onClick={onRemove}
        tooltipText={t("Remove item")}
      />
    </div>
  );
};
