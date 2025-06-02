import { ChevronDownIcon } from "@radix-ui/react-icons";
import {
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  WidgetProps,
} from "@rjsf/utils";
import { useState } from "react";

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
  value,
  onChange,
  onBlur,
  onFocus,
  placeholder,
  rawErrors = [],
}: WidgetProps<T, S, F>) => {
  const { enumOptions, enumDisabled } = options;
  const [selectedLabel, setSelectedLabel] = useState(placeholder);

  const getCurrentLabel = () => {
    const option = enumOptions?.find((opt: any) => opt.value === value);
    return option ? option.label : placeholder;
  };

  const handleSelect = (value: any, label: string) => {
    setSelectedLabel(label);
    onChange(value);
  };

  const handleBlur = () => onBlur?.(id, value);
  const handleFocus = () => onFocus?.(id, value);

  return (
    <DropdownMenu modal={true}>
      <DropdownMenuTrigger
        className={`flex h-8 w-full items-center justify-between rounded border bg-background px-3 hover:bg-accent ${
          rawErrors.length > 0 ? "border-destructive" : ""
        }`}
        disabled={readonly || disabled}
        onBlur={handleBlur}
        onFocus={handleFocus}>
        <span className={`${value ? "" : "text-muted-foreground"}`}>
          {selectedLabel || getCurrentLabel()}
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
              onSelect={() => handleSelect(optionValue, label)}
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
