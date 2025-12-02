import {
  ariaDescribedByIds,
  enumOptionsIsSelected,
  // enumOptionsValueForIndex,
  optionId,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  WidgetProps,
} from "@rjsf/utils";
// import { ChangeEvent, FocusEvent } from "react";

import { RadioGroup, Label, RadioGroupItem } from "@flow/components";

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
  // onChange,
  // onBlur,
  // onFocus,
}: WidgetProps<T, S, F>) => {
  const {
    enumOptions,
    enumDisabled,
    // emptyValue
  } = options;

  // const _onChange = ({ target: { value } }: ChangeEvent<HTMLInputElement>) =>
  //   onChange(enumOptionsValueForIndex<S>(value, enumOptions, emptyValue));
  // const _onBlur = ({ target }: FocusEvent<HTMLInputElement>) =>
  //   onBlur(
  //     id,
  //     enumOptionsValueForIndex<S>(
  //       target && target.value,
  //       enumOptions,
  //       emptyValue,
  //     ),
  //   );
  // const _onFocus = ({ target }: FocusEvent<HTMLInputElement>) =>
  //   onFocus(
  //     id,
  //     enumOptionsValueForIndex<S>(
  //       target && target.value,
  //       enumOptions,
  //       emptyValue,
  //     ),
  //   );

  return (
    <RadioGroup>
      {Array.isArray(enumOptions) &&
        enumOptions.map((option, index) => {
          const itemDisabled =
            Array.isArray(enumDisabled) &&
            enumDisabled.indexOf(option.value) !== -1;
          const checked = enumOptionsIsSelected<S>(option.value, value);

          return (
            <div className="flex items-center space-x-2">
              <RadioGroupItem
                key={optionId(id, index)}
                // label={option.label}
                id={optionId(id, index)}
                // name={id}
                disabled={readonly || itemDisabled || disabled}
                checked={checked}
                required={required}
                value={String(index)}
                // TODO: Fix radio group
                // onChange={_onChange}
                // onBlur={_onBlur}
                // onFocus={_onFocus}
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
