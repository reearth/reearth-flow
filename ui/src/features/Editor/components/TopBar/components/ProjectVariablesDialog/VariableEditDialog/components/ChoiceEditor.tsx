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
  RadioGroup,
  RadioGroupItem,
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
import { AnyProjectVariable, ChoiceConfig } from "@flow/types";

type Props = {
  variable: AnyProjectVariable;
  assetUrl?: string | null;
  cmsItemAssetUrl?: string | null;
  onUpdate: (variable: AnyProjectVariable) => void;
  onDialogOpen?: (dialog: "assets" | "cms") => void;
  clearUrl: () => void;
};

export const ChoiceEditor: React.FC<Props> = ({
  variable,
  assetUrl,
  onUpdate,
  onDialogOpen,
  clearUrl,
}) => {
  const t = useT();

  // Get the current choice configuration from the config field
  const getChoiceConfig = useCallback((): ChoiceConfig => {
    if (variable.type === "choice" && variable.config) {
      return variable.config as ChoiceConfig;
    }

    // Default fallback
    return {
      choices: ["Option 1", "Option 2", "Option 3"],
      displayMode: "dropdown",
      allowMultiple: false,
    };
  }, [variable.config, variable.type]);

  const [choiceConfig, setChoiceConfig] =
    useState<ChoiceConfig>(getChoiceConfig());
  const [newOptionText, setNewOptionText] = useState("");

  // Sync config when variable changes
  useEffect(() => {
    setChoiceConfig(getChoiceConfig());
  }, [getChoiceConfig]);

  useEffect(() => {
    if (assetUrl) {
      setNewOptionText(assetUrl);
      clearUrl();
    }
  }, [assetUrl, clearUrl]);

  const updateVariable = (
    config: ChoiceConfig,
    selectedOption?: string | string[],
  ) => {
    setChoiceConfig(config);

    const updatedVariable: AnyProjectVariable = {
      ...variable,
      config: variable.type === "choice" ? config : variable.config,
      defaultValue:
        selectedOption !== undefined ? selectedOption : variable.defaultValue,
    };

    onUpdate(updatedVariable);
  };

  const handleAddOption = () => {
    if (
      newOptionText.trim() &&
      !choiceConfig.choices.includes(newOptionText.trim())
    ) {
      const newConfig = {
        ...choiceConfig,
        choices: [...choiceConfig.choices, newOptionText.trim()],
      };
      updateVariable(newConfig);
      setNewOptionText("");
    }
  };

  const handleRemoveOption = (index: number) => {
    const removedOption = choiceConfig.choices[index];
    const newChoices = choiceConfig.choices.filter((_, i) => i !== index);
    const newConfig = {
      ...choiceConfig,
      choices: newChoices,
    };

    // Update selected option if it was the removed one
    const currentSelected =
      typeof variable.defaultValue === "string" ? variable.defaultValue : "";
    const newSelected =
      currentSelected === removedOption
        ? newChoices.length > 0
          ? newChoices[0]
          : ""
        : currentSelected;

    updateVariable(newConfig, newSelected);
  };

  // Set up drag and drop sensors
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  );

  // Handle drag end for reordering options
  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (active.id !== over?.id) {
      const oldIndex = choiceConfig.choices.findIndex(
        (option, index) => `${option}-${index}` === active.id,
      );
      const newIndex = choiceConfig.choices.findIndex(
        (option, index) => `${option}-${index}` === over?.id,
      );

      if (oldIndex !== -1 && newIndex !== -1) {
        const newChoices = [...choiceConfig.choices];
        const [movedOption] = newChoices.splice(oldIndex, 1);
        newChoices.splice(newIndex, 0, movedOption);

        updateVariable({
          ...choiceConfig,
          choices: newChoices,
        });
      }
    }
  };

  const handleUpdateOption = (index: number, newText: string) => {
    const newChoices = [...choiceConfig.choices];
    const oldOption = newChoices[index];
    newChoices[index] = newText;

    const newConfig = {
      ...choiceConfig,
      choices: newChoices,
    };

    // Update selected option if it was the old option
    const currentSelected =
      typeof variable.defaultValue === "string" ? variable.defaultValue : "";
    const newSelected =
      currentSelected === oldOption ? newText : currentSelected;

    updateVariable(newConfig, newSelected);
  };

  const handleSelectDefault = (option: string | string[]) => {
    updateVariable(choiceConfig, option);
  };

  const handleDisplayModeChange = (mode: "dropdown" | "radio") => {
    const newConfig = {
      ...choiceConfig,
      displayMode: mode,
    };
    updateVariable(newConfig);
  };

  const handleAllowMultipleChange = (allowMultiple: boolean) => {
    const newConfig = {
      ...choiceConfig,
      allowMultiple,
    };
    // Reset default value when switching between single/multiple
    const newDefaultValue = allowMultiple ? [] : "";
    updateVariable(newConfig, newDefaultValue);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleAddOption();
    }
  };

  // Helper functions for multiple selection handling
  const getCurrentSelection = (): string[] => {
    if (choiceConfig.allowMultiple) {
      return Array.isArray(variable.defaultValue) ? variable.defaultValue : [];
    }
    return typeof variable.defaultValue === "string" && variable.defaultValue
      ? [variable.defaultValue]
      : [];
  };

  const handleMultipleSelection = (option: string, checked: boolean) => {
    const currentSelection = getCurrentSelection();
    let newSelection: string[];

    if (checked) {
      newSelection = [...currentSelection, option];
    } else {
      newSelection = currentSelection.filter((item) => item !== option);
    }

    handleSelectDefault(
      choiceConfig.allowMultiple ? newSelection : newSelection[0] || "",
    );
  };

  return (
    <div className="space-y-6">
      {/* Configuration Options */}
      <div>
        <h3 className="mb-4 text-lg font-medium">{t("Configuration")}</h3>

        <div className="mb-6 grid grid-cols-2 gap-4">
          <div>
            <Label className="mb-2 block text-sm font-medium">
              {t("Display Mode")}
            </Label>
            <Select
              value={choiceConfig.displayMode || "dropdown"}
              onValueChange={(value: "dropdown" | "radio") =>
                handleDisplayModeChange(value)
              }>
              <SelectTrigger>
                <SelectValue placeholder={t("Select display mode")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="dropdown">{t("Dropdown")}</SelectItem>
                <SelectItem value="radio">{t("Radio Buttons")}</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div>
            <Label className="mb-2 block text-sm font-medium">
              {t("Allow Multiple")}
            </Label>
            <div className="mt-2 flex items-center space-x-2">
              <Switch
                checked={choiceConfig.allowMultiple || false}
                onCheckedChange={handleAllowMultipleChange}
              />
              <span className="text-sm text-muted-foreground">
                {choiceConfig.allowMultiple
                  ? t("Multiple selections allowed")
                  : t("Single selection only")}
              </span>
            </div>
          </div>
        </div>
      </div>

      <div>
        <h3 className="mb-4 text-lg font-medium">{t("Choice Options")}</h3>

        {/* Add new option */}
        <div className="mb-4 flex gap-2">
          <Input
            value={newOptionText}
            onChange={(e) => setNewOptionText(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t("Enter new option")}
            className="flex-1"
          />
          {onDialogOpen && (
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
            onClick={handleAddOption}
            disabled={
              !newOptionText.trim() ||
              choiceConfig.choices.includes(newOptionText.trim())
            }
            tooltipText={t("Add option")}
          />
        </div>

        {/* Options list */}
        <div className="space-y-2">
          <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragEnd={handleDragEnd}>
            <SortableContext
              items={choiceConfig.choices.map(
                (option, index) => `${option}-${index}`,
              )}
              strategy={verticalListSortingStrategy}>
              {choiceConfig.choices.map((option, index) => (
                <SortableOptionRow
                  key={`${option}-${index}`}
                  option={option}
                  index={index}
                  onUpdate={(newText) => handleUpdateOption(index, newText)}
                  onRemove={() => handleRemoveOption(index)}
                />
              ))}
            </SortableContext>
          </DndContext>
        </div>
      </div>

      {/* Default selection */}
      {choiceConfig.choices.length > 0 && (
        <div>
          <h3 className="mb-4 text-lg font-medium">{t("Default Selection")}</h3>

          {choiceConfig.allowMultiple ? (
            // Multiple selection with checkboxes
            <div className="space-y-2">
              <p className="mb-3 text-sm text-muted-foreground">
                {t("Select default options (multiple allowed)")}
              </p>
              {choiceConfig.choices.map((option, index) => {
                const currentSelection = getCurrentSelection();
                const isChecked = currentSelection.includes(option);

                return (
                  <div
                    key={`checkbox-${option}-${index}`}
                    className="flex items-center space-x-2">
                    <Checkbox
                      id={`default-option-${index}`}
                      checked={isChecked}
                      onCheckedChange={(checked) =>
                        handleMultipleSelection(option, !!checked)
                      }
                    />
                    <Label htmlFor={`default-option-${index}`}>{option}</Label>
                  </div>
                );
              })}
            </div>
          ) : (
            // Single selection with radio buttons
            <RadioGroup
              value={
                typeof variable.defaultValue === "string"
                  ? variable.defaultValue
                  : ""
              }
              onValueChange={handleSelectDefault}>
              <div className="flex items-center space-x-2">
                <RadioGroupItem value="" id="no-default" />
                <Label htmlFor="no-default" className="text-muted-foreground">
                  {t("No default")}
                </Label>
              </div>
              {choiceConfig.choices.map((option, index) => (
                <div
                  key={`radio-${option}-${index}`}
                  className="flex items-center space-x-2">
                  <RadioGroupItem value={option} id={`option-${index}`} />
                  <Label htmlFor={`option-${index}`}>{option}</Label>
                </div>
              ))}
            </RadioGroup>
          )}
        </div>
      )}
    </div>
  );
};

// Sortable option row component
const SortableOptionRow: React.FC<{
  option: string;
  index: number;
  onUpdate: (newText: string) => void;
  onRemove: () => void;
}> = ({ option, index, onUpdate, onRemove }) => {
  const t = useT();
  const [localValue, setLocalValue] = useState(option);

  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: `${option}-${index}`,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  // Sync local value when option prop changes (due to reordering)
  useEffect(() => {
    setLocalValue(option);
  }, [option]);

  const handleBlur = () => {
    if (localValue !== option && localValue.trim()) {
      onUpdate(localValue.trim());
    } else {
      setLocalValue(option);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.currentTarget.blur();
    } else if (e.key === "Escape") {
      setLocalValue(option);
    }
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

      <Input
        value={localValue}
        onChange={(e) => setLocalValue(e.target.value)}
        onBlur={handleBlur}
        onKeyDown={handleKeyDown}
        className="flex-1"
      />

      <IconButton
        icon={<TrashIcon size={14} />}
        size="sm"
        variant="ghost"
        onClick={onRemove}
        tooltipText={t("Remove option")}
      />
    </div>
  );
};
