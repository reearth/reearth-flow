import { CaretDownIcon } from "@phosphor-icons/react";
import {
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  WidgetProps,
} from "@rjsf/utils";
import { useCallback } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";

import { NULL_BRANCH_TITLE } from "../patchSchemaTypes";
import { paramsAwarenessStyles } from "../utils/awarenessTemplateStyles";

const SelectWidget = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  id,
  options,
  disabled,
  readonly,
  required,
  registry,
  value,
  multiple,
  onChange,
  onBlur,
  onFocus,
  placeholder,
  rawErrors = [],
}: WidgetProps<T, S, F>) => {
  const { enumOptions, enumDisabled, emptyValue } = options;
  const formContext = registry?.formContext;
  const { fieldFocusMap, onFieldFocus } = formContext ?? {};
  const focusedUsers = fieldFocusMap?.[id] ?? [];
  const showPlaceholderOption = !multiple && !required;

  const getCurrentLabel = useCallback(() => {
    const option = enumOptions?.find((opt: any) => opt.value === value);
    return option ? option.label : placeholder;
  }, [enumOptions, value, placeholder]);

  // A schema may carry its own "not set" branch — the trigger's empty state
  // already says that, so it must not also appear as an item to pick.
  const selectableOptions = enumOptions?.filter(
    (opt: any) => opt.label !== NULL_BRANCH_TITLE,
  );

  // `0` is a real selection, not an empty one: RJSF identifies the branch of an
  // xxxOf by its index, so the first variant arrives here as a falsy value and a
  // plain truthiness test would grey it out as though nothing were chosen.
  // Landing on the "not set" branch is the one case that really is empty.
  const hasValue =
    value !== undefined &&
    value !== null &&
    value !== "" &&
    getCurrentLabel() !== NULL_BRANCH_TITLE;

  const handleSelect = useCallback(
    (selectedValue: any) => {
      onChange(selectedValue);
    },
    [onChange],
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
    <DropdownMenu modal={true}>
      <DropdownMenuTrigger
        className={`flex h-8 max-w-141 min-w-[30%] items-center justify-between gap-2 rounded border bg-background px-3 hover:bg-accent ${
          rawErrors.length > 0 ? "border-destructive" : ""
        }`}
        style={paramsAwarenessStyles(focusedUsers)}
        disabled={readonly || disabled}
        onBlur={handleBlur}
        onFocus={handleFocus}
        aria-label={placeholder || "Select an option"}
        aria-required={required}
        aria-invalid={rawErrors.length > 0}
        aria-describedby={rawErrors.length > 0 ? `${id}-error` : undefined}>
        <span className={`${hasValue ? "" : "text-muted-foreground"}`}>
          {getCurrentLabel() || placeholder || "-"}
        </span>
        <CaretDownIcon className="size-4" />
      </DropdownMenuTrigger>
      <DropdownMenuContent className="max-h-60 overflow-auto" align="start">
        {showPlaceholderOption && (
          <DropdownMenuItem
            onClick={() => handleSelect(emptyValue)}
            className={`text-muted-foreground ${value == null ? "bg-accent" : ""}`}>
            {placeholder || "-"}
          </DropdownMenuItem>
        )}
        {selectableOptions?.map(
          ({ value: optionValue, label }: any, i: number) => {
            const isDisabled = enumDisabled?.includes(optionValue);
            return (
              <DropdownMenuItem
                key={i}
                disabled={isDisabled}
                onClick={() => handleSelect(optionValue)}
                className={`${value === optionValue ? "bg-accent" : ""}`}>
                {label}
              </DropdownMenuItem>
            );
          },
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export { SelectWidget };
