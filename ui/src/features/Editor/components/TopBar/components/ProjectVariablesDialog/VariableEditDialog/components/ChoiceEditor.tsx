import {
  CaretDownIcon,
  CaretUpIcon,
  MinusIcon,
  PlusIcon,
} from "@phosphor-icons/react";
import { useState, useEffect, useCallback } from "react";

import {
  Input,
  IconButton,
  RadioGroup,
  RadioGroupItem,
  Label,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { ProjectVariable } from "@flow/types";

type ChoiceConfig = {
  options: string[];
  selectedOption?: string;
};

type Props = {
  variable: ProjectVariable;
  onUpdate: (variable: ProjectVariable) => void;
};

export const ChoiceEditor: React.FC<Props> = ({ variable, onUpdate }) => {
  const t = useT();

  // Parse the current choice configuration
  const getChoiceConfig = useCallback((): ChoiceConfig => {
    if (
      typeof variable.defaultValue === "object" &&
      variable.defaultValue?.options
    ) {
      return variable.defaultValue as ChoiceConfig;
    }
    // Fallback for old string-based choices
    if (typeof variable.defaultValue === "string") {
      return {
        options: ["Option 1", "Option 2", "Option 3"],
        selectedOption: variable.defaultValue || undefined,
      };
    }
    return {
      options: ["Option 1", "Option 2", "Option 3"],
      selectedOption: undefined,
    };
  }, [variable.defaultValue]);

  const [choiceConfig, setChoiceConfig] =
    useState<ChoiceConfig>(getChoiceConfig());
  const [newOptionText, setNewOptionText] = useState("");

  // Sync config when variable changes
  useEffect(() => {
    setChoiceConfig(getChoiceConfig());
  }, [getChoiceConfig]);

  const updateVariable = (config: ChoiceConfig) => {
    setChoiceConfig(config);
    onUpdate({
      ...variable,
      defaultValue: config,
    });
  };

  const handleAddOption = () => {
    if (
      newOptionText.trim() &&
      !choiceConfig.options.includes(newOptionText.trim())
    ) {
      const newConfig = {
        ...choiceConfig,
        options: [...choiceConfig.options, newOptionText.trim()],
      };
      updateVariable(newConfig);
      setNewOptionText("");
    }
  };

  const handleRemoveOption = (index: number) => {
    const removedOption = choiceConfig.options[index];
    const newOptions = choiceConfig.options.filter((_, i) => i !== index);
    const newConfig = {
      ...choiceConfig,
      options: newOptions,
      selectedOption:
        choiceConfig.selectedOption === removedOption
          ? newOptions.length > 0
            ? newOptions[0]
            : undefined
          : choiceConfig.selectedOption,
    };
    updateVariable(newConfig);
  };

  const handleMoveOption = (index: number, direction: "up" | "down") => {
    const newOptions = [...choiceConfig.options];
    const targetIndex = direction === "up" ? index - 1 : index + 1;

    if (targetIndex >= 0 && targetIndex < newOptions.length) {
      [newOptions[index], newOptions[targetIndex]] = [
        newOptions[targetIndex],
        newOptions[index],
      ];
      updateVariable({
        ...choiceConfig,
        options: newOptions,
      });
    }
  };

  const handleUpdateOption = (index: number, newText: string) => {
    const newOptions = [...choiceConfig.options];
    const oldOption = newOptions[index];
    newOptions[index] = newText;

    const newConfig = {
      ...choiceConfig,
      options: newOptions,
      selectedOption:
        choiceConfig.selectedOption === oldOption
          ? newText
          : choiceConfig.selectedOption,
    };
    updateVariable(newConfig);
  };

  const handleSelectDefault = (option: string) => {
    updateVariable({
      ...choiceConfig,
      selectedOption: option === "" ? undefined : option,
    });
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleAddOption();
    }
  };

  return (
    <div className="space-y-6">
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
          <IconButton
            icon={<PlusIcon />}
            onClick={handleAddOption}
            disabled={
              !newOptionText.trim() ||
              choiceConfig.options.includes(newOptionText.trim())
            }
            tooltipText={t("Add option")}
          />
        </div>

        {/* Options list */}
        <div className="space-y-2">
          {choiceConfig.options.map((option, index) => (
            <OptionRow
              key={`${option}-${index}`}
              option={option}
              index={index}
              isFirst={index === 0}
              isLast={index === choiceConfig.options.length - 1}
              onUpdate={(newText) => handleUpdateOption(index, newText)}
              onRemove={() => handleRemoveOption(index)}
              onMoveUp={() => handleMoveOption(index, "up")}
              onMoveDown={() => handleMoveOption(index, "down")}
            />
          ))}
        </div>
      </div>

      {/* Default selection */}
      {choiceConfig.options.length > 0 && (
        <div>
          <h3 className="mb-4 text-lg font-medium">{t("Default Selection")}</h3>
          <RadioGroup
            value={choiceConfig.selectedOption ?? ""}
            onValueChange={handleSelectDefault}>
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="" id="no-default" />
              <Label htmlFor="no-default" className="text-muted-foreground">
                {t("No default")}
              </Label>
            </div>
            {choiceConfig.options.map((option, index) => (
              <div
                key={`radio-${option}-${index}`}
                className="flex items-center space-x-2">
                <RadioGroupItem value={option} id={`option-${index}`} />
                <Label htmlFor={`option-${index}`}>{option}</Label>
              </div>
            ))}
          </RadioGroup>
        </div>
      )}
    </div>
  );
};

// Individual option row component
const OptionRow: React.FC<{
  option: string;
  index: number;
  isFirst: boolean;
  isLast: boolean;
  onUpdate: (newText: string) => void;
  onRemove: () => void;
  onMoveUp: () => void;
  onMoveDown: () => void;
}> = ({
  option,
  index,
  isFirst,
  isLast,
  onUpdate,
  onRemove,
  onMoveUp,
  onMoveDown,
}) => {
  const t = useT();
  const [localValue, setLocalValue] = useState(option);

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
    <div className="flex items-center gap-2 rounded-md border p-2">
      <span className="w-8 text-sm text-muted-foreground">{index + 1}.</span>

      <Input
        value={localValue}
        onChange={(e) => setLocalValue(e.target.value)}
        onBlur={handleBlur}
        onKeyDown={handleKeyDown}
        className="flex-1"
      />

      <div className="flex gap-1">
        <IconButton
          icon={<CaretUpIcon size={14} />}
          size="sm"
          variant="ghost"
          disabled={isFirst}
          onClick={onMoveUp}
          tooltipText={t("Move up")}
        />
        <IconButton
          icon={<CaretDownIcon size={14} />}
          size="sm"
          variant="ghost"
          disabled={isLast}
          onClick={onMoveDown}
          tooltipText={t("Move down")}
        />
        <IconButton
          icon={<MinusIcon size={14} />}
          size="sm"
          variant="ghost"
          onClick={onRemove}
          tooltipText={t("Remove option")}
        />
      </div>
    </div>
  );
};
