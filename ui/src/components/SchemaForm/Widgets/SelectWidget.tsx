import { ChevronDownIcon } from "@radix-ui/react-icons";
import {
  ariaDescribedByIds,
  FormContextType,
  // enumOptionsIndexForValue,
  // enumOptionsValueForIndex,
  RJSFSchema,
  StrictRJSFSchema,
  WidgetProps,
} from "@rjsf/utils";
// import { ChangeEvent, FocusEvent } from "react";

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
  // required,
  // disabled,
  // readonly,
  // value,
  // multiple,
  // autofocus,
  // onChange,
  // onBlur,
  // onFocus,
  placeholder,
  // rawErrors = [],
}: WidgetProps<T, S, F>) => {
  const {
    enumOptions,
    enumDisabled,
    // emptyValue: optEmptyValue
  } = options;

  // const emptyValue = multiple ? [] : "";

  // function getValue(event: FocusEvent | ChangeEvent | any, multiple?: boolean) {
  //   if (multiple) {
  //     return [].slice
  //       .call(event.target.options as any)
  //       .filter((o: any) => o.selected)
  //       .map((o: any) => o.value);
  //   } else {
  //     return event.target.value;
  //   }
  // }
  // const selectedIndexes = enumOptionsIndexForValue<S>(
  //   value,
  //   enumOptions,
  //   multiple,
  // );

  return (
    <DropdownMenu
      modal={true}
      // id={id}
      // name={id}
      // value={
      //   typeof selectedIndexes === "undefined" ? emptyValue : selectedIndexes
      // }
      // required={required}
      // multiple={multiple}
      // disabled={disabled || readonly}
      // autoFocus={autofocus}
      // className={rawErrors.length > 0 ? "text-destructive" : ""}
      // onBlur={
      //   onBlur &&
      //   ((event: FocusEvent) => {
      //     const newValue = getValue(event, multiple);
      //     onBlur(
      //       id,
      //       enumOptionsValueForIndex<S>(newValue, enumOptions, optEmptyValue),
      //     );
      //   })
      // }
      // onFocus={
      //   onFocus &&
      //   ((event: FocusEvent) => {
      //     const newValue = getValue(event, multiple);
      //     onFocus(
      //       id,
      //       enumOptionsValueForIndex<S>(newValue, enumOptions, optEmptyValue),
      //     );
      //   })
      // }
      // onChange={(event: ChangeEvent) => {
      //   const newValue = getValue(event, multiple);
      //   onChange(
      //     enumOptionsValueForIndex<S>(newValue, enumOptions, optEmptyValue),
      //   );
      // }}
      aria-describedby={ariaDescribedByIds<T>(id)}>
      <DropdownMenuTrigger className="flex h-8 items-center rounded border bg-background px-1 hover:bg-accent">
        <p> {placeholder}</p>
        <ChevronDownIcon />
      </DropdownMenuTrigger>
      <DropdownMenuContent className="overflow-auto" align="center">
        {(enumOptions as any).map(({ value, label }: any, i: number) => {
          const disabled: any =
            Array.isArray(enumDisabled) &&
            (enumDisabled as any).indexOf(value) != -1;
          return (
            <DropdownMenuItem
              key={i}
              id={label}
              // value={String(i)}
              disabled={disabled}>
              {label}
            </DropdownMenuItem>
          );
        })}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export { SelectWidget };
