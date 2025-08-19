import { ChevronDownIcon } from "@radix-ui/react-icons";
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
  value,
  onChange,
  onBlur,
  onFocus,
  placeholder,
  rawErrors = [],
}: WidgetProps<T, S, F>) => {
  const { enumOptions, enumDisabled } = options;

  const getCurrentLabel = useCallback(() => {
    const option = enumOptions?.find((opt: any) => opt.value === value);
    return option ? option.label : placeholder;
  }, [enumOptions, value, placeholder]);

  const handleSelect = useCallback(
    (selectedValue: any) => {
      onChange(selectedValue);
    },
    [onChange],
  );

  const handleBlur = useCallback(
    () => onBlur?.(id, value),
    [onBlur, id, value],
  );
  const handleFocus = useCallback(
    () => onFocus?.(id, value),
    [onFocus, id, value],
  );

  return (
    <DropdownMenu modal={true}>
      <DropdownMenuTrigger
        className={`flex h-8 max-w-[564px] items-center justify-between gap-2 rounded border bg-background px-3 hover:bg-accent ${
          rawErrors.length > 0 ? "border-destructive" : ""
        }`}
        disabled={readonly || disabled}
        onBlur={handleBlur}
        onFocus={handleFocus}
        aria-label={placeholder || "Select an option"}
        aria-required={required}
        aria-invalid={rawErrors.length > 0}
        aria-describedby={rawErrors.length > 0 ? `${id}-error` : undefined}>
        <span className={`${value ? "" : "text-muted-foreground"}`}>
          {getCurrentLabel() || placeholder}
        </span>
        <ChevronDownIcon className="size-4" />
      </DropdownMenuTrigger>
      <DropdownMenuContent className="max-h-60 overflow-auto" align="start">
        {enumOptions?.map(({ value: optionValue, label }: any, i: number) => {
          const isDisabled = enumDisabled?.includes(optionValue);
          return (
            <DropdownMenuItem
              key={i}
              disabled={isDisabled}
              onSelect={() => handleSelect(optionValue)}
              className={`${value === optionValue ? "bg-accent" : ""}`}>
              {label}
            </DropdownMenuItem>
          );
        })}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export { SelectWidget };
