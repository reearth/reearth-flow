import {
  ariaDescribedByIds,
  enumOptionsIsSelected,
  enumOptionsValueForIndex,
  optionId,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  WidgetProps,
} from "@rjsf/utils";
import { useCallback } from "react";

import { RadioGroup, Label, RadioGroupItem } from "@flow/components";

import { paramsAwarenessStyles } from "../utils/awarenessTemplateStyles";

const RadioWidget = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  id,
  options,
  value,
  required,
  disabled,
  readonly,
  registry,
  onChange,
  onBlur,
  onFocus,
}: WidgetProps<T, S, F>) => {
  const { enumOptions, enumDisabled, emptyValue } = options;
  const formContext = registry?.formContext;
  const { fieldFocusMap, onFieldFocus } = formContext ?? {};
  const focusedUsers = fieldFocusMap?.[id] ?? [];

  const selectedIndex = Array.isArray(enumOptions)
    ? enumOptions.findIndex((option) =>
        enumOptionsIsSelected<S>(option.value, value),
      )
    : -1;

  const handleValueChange = useCallback(
    (newValue: unknown) =>
      onChange(
        enumOptionsValueForIndex<S>(
          newValue as string,
          enumOptions,
          emptyValue,
        ),
      ),
    [onChange, enumOptions, emptyValue],
  );

  const handleBlur = useCallback(() => {
    onBlur?.(id, value);
    onFieldFocus?.(null);
  }, [onBlur, onFieldFocus, id, value]);

  const handleFocus = useCallback(() => {
    onFocus?.(id, value);
    onFieldFocus?.(id);
  }, [onFocus, onFieldFocus, id, value]);

  return (
    <RadioGroup
      value={selectedIndex >= 0 ? String(selectedIndex) : ""}
      disabled={readonly || disabled}
      style={paramsAwarenessStyles(focusedUsers)}
      onValueChange={handleValueChange}
      onBlur={handleBlur}
      onFocus={handleFocus}>
      {Array.isArray(enumOptions) &&
        enumOptions.map((option, index) => {
          const itemDisabled =
            Array.isArray(enumDisabled) &&
            enumDisabled.indexOf(option.value) !== -1;

          return (
            <div
              key={optionId(id, index)}
              className="flex items-center space-x-2">
              <RadioGroupItem
                id={optionId(id, index)}
                disabled={readonly || itemDisabled || disabled}
                required={required}
                value={String(index)}
                aria-describedby={ariaDescribedByIds(id)}
              />
              <Label htmlFor={optionId(id, index)}>{option.label}</Label>
            </div>
          );
        })}
    </RadioGroup>
  );
};

export { RadioWidget };
