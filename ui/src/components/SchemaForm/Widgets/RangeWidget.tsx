import {
  ariaDescribedByIds,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  WidgetProps,
} from "@rjsf/utils";
import { ChangeEvent, useCallback, useState } from "react";

import { Button, Input, Label } from "@flow/components";
import { useT } from "@flow/lib/i18n";

const RangeWidget = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  id,
  value,
  disabled,
  readonly,
  onChange,
  onBlur,
  onFocus,
  schema,
  rawErrors = [],
}: WidgetProps<T, S, F>) => {
  const t = useT();
  const [isEditing, setIsEditing] = useState(false);
  const [inputValue, setInputValue] = useState(String(value || ""));

  // Extract range properties from schema
  const min = schema.minimum ?? 0;
  const max = schema.maximum ?? 100;
  const step = schema.multipleOf ?? 1;
  const defaultValue = schema.default ?? min;

  // Ensure value is within bounds
  const normalizedValue = Math.max(min, Math.min(max, Number(value) || min));

  const handleSliderChange = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => {
      const newValue = Number(e.target.value);
      onChange(newValue);
    },
    [onChange],
  );

  const handleInputChange = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    setInputValue(e.target.value);
  }, []);

  const handleInputSubmit = useCallback(() => {
    const numericValue = Number(inputValue);
    if (!isNaN(numericValue)) {
      const boundedValue = Math.max(min, Math.min(max, numericValue));
      onChange(boundedValue);
      setInputValue(String(boundedValue));
    } else {
      setInputValue(String(normalizedValue));
    }
    setIsEditing(false);
  }, [inputValue, min, max, onChange, normalizedValue]);

  const handleInputKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter") {
        handleInputSubmit();
      } else if (e.key === "Escape") {
        setInputValue(String(normalizedValue));
        setIsEditing(false);
      }
    },
    [handleInputSubmit, normalizedValue],
  );

  const handleReset = useCallback(() => {
    onChange(defaultValue);
  }, [onChange, defaultValue]);

  const percentage = ((normalizedValue - min) / (max - min)) * 100;

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-4">
        {/* Range Slider */}
        <div className="relative flex-1">
          <Input
            id={id}
            type="range"
            min={min}
            max={max}
            step={step}
            value={normalizedValue}
            disabled={readonly || disabled}
            onChange={handleSliderChange}
            onBlur={(e) => onBlur?.(id, Number(e.target.value))}
            onFocus={(e) => onFocus?.(id, Number(e.target.value))}
            aria-describedby={ariaDescribedByIds(id)}
            aria-invalid={rawErrors.length > 0}
            className="h-2 cursor-pointer appearance-none rounded-lg bg-secondary"
            style={{
              background: `linear-gradient(to right, hsl(var(--primary)) 0%, hsl(var(--primary)) ${percentage}%, hsl(var(--secondary)) ${percentage}%, hsl(var(--secondary)) 100%)`,
            }}
          />
        </div>

        {/* Value Display/Input */}
        <div className="flex items-center gap-2">
          {isEditing ? (
            <div className="flex items-center gap-1">
              <Input
                type="number"
                value={inputValue}
                onChange={handleInputChange}
                onKeyDown={handleInputKeyDown}
                onBlur={handleInputSubmit}
                min={min}
                max={max}
                step={step}
                className="w-20 text-center"
                autoFocus
              />
            </div>
          ) : (
            <button
              type="button"
              onClick={() => {
                setIsEditing(true);
                setInputValue(String(normalizedValue));
              }}
              disabled={readonly || disabled}
              className="min-w-[60px] rounded border px-2 py-1 text-center text-sm hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
              aria-label={`Current value: ${normalizedValue}. Click to edit.`}>
              {normalizedValue}
            </button>
          )}

          {/* Reset Button */}
          {normalizedValue !== defaultValue && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={handleReset}
              disabled={readonly || disabled}
              className="h-8 px-2"
              aria-label={`Reset to default: ${defaultValue}`}>
              {t("Reset")}
            </Button>
          )}
        </div>
      </div>

      {/* Range Labels */}
      <div className="flex justify-between text-xs text-muted-foreground">
        <Label>{min}</Label>
        <Label>{max}</Label>
      </div>

      {/* Error Display */}
      {rawErrors.length > 0 &&
        rawErrors.map((error, i) => (
          <p key={i} className="text-xs text-destructive" role="alert">
            {error}
          </p>
        ))}
    </div>
  );
};

export { RangeWidget };
