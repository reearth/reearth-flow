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
import { AnyProjectVariable, ChoiceConfig } from "@flow/types";

type Props = {
  variable: AnyProjectVariable;
  onUpdate: (variable: AnyProjectVariable) => void;
};

export const ChoiceEditor: React.FC<Props> = ({ variable, onUpdate }) => {
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
    };
  }, [variable.config, variable.type]);

  const [choiceConfig, setChoiceConfig] =
    useState<ChoiceConfig>(getChoiceConfig());
  const [newOptionText, setNewOptionText] = useState("");

  // Sync config when variable changes
  useEffect(() => {
    setChoiceConfig(getChoiceConfig());
  }, [getChoiceConfig]);

  const updateVariable = (config: ChoiceConfig, selectedOption?: string) => {
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

  const handleMoveOption = (index: number, direction: "up" | "down") => {
    const newChoices = [...choiceConfig.choices];
    const targetIndex = direction === "up" ? index - 1 : index + 1;

    if (targetIndex >= 0 && targetIndex < newChoices.length) {
      [newChoices[index], newChoices[targetIndex]] = [
        newChoices[targetIndex],
        newChoices[index],
      ];
      updateVariable({
        ...choiceConfig,
        choices: newChoices,
      });
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

  const handleSelectDefault = (option: string) => {
    updateVariable(choiceConfig, option === "" ? "" : option);
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
              choiceConfig.choices.includes(newOptionText.trim())
            }
            tooltipText={t("Add option")}
          />
        </div>

        {/* Options list */}
        <div className="space-y-2">
          {choiceConfig.choices.map((option, index) => (
            <OptionRow
              key={`${option}-${index}`}
              option={option}
              index={index}
              isFirst={index === 0}
              isLast={index === choiceConfig.choices.length - 1}
              onUpdate={(newText) => handleUpdateOption(index, newText)}
              onRemove={() => handleRemoveOption(index)}
              onMoveUp={() => handleMoveOption(index, "up")}
              onMoveDown={() => handleMoveOption(index, "down")}
            />
          ))}
        </div>
      </div>

      {/* Default selection */}
      {choiceConfig.choices.length > 0 && (
        <div>
          <h3 className="mb-4 text-lg font-medium">{t("Default Selection")}</h3>
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
