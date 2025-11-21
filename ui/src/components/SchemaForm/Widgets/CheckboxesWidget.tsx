import {
  ariaDescribedByIds,
  // enumOptionsDeselectValue,
  enumOptionsIsSelected,
  // enumOptionsSelectValue,
  enumOptionsValueForIndex,
  optionId,
  FormContextType,
  WidgetProps,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";
import {
  // ChangeEvent,
  FocusEvent,
} from "react";

import { Checkbox } from "@flow/components";

/** The `CheckboxesWidget` is a widget for rendering checkbox groups.
 *  It is typically used to represent an array of enums.
 *
 * @param props - The `WidgetProps` for this component
 */
const CheckboxesWidget = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  label,
  hideLabel,
  id,
  disabled,
  options,
  value,
  autofocus,
  readonly,
  //   TODO: Fix this required
  //   required,
  // onChange,
  onBlur,
  onFocus,
}: WidgetProps<T, S, F>) => {
  const { enumOptions, enumDisabled, emptyValue } = options;
  const checkboxesValues = Array.isArray(value) ? value : [value];

  // const _onChange =
  //   (index: number) =>
  //   ({ target: { checked } }: ChangeEvent<HTMLInputElement>) => {
  //     if (checked) {
  //       onChange(enumOptionsSelectValue(index, checkboxesValues, enumOptions));
  //     } else {
  //       onChange(
  //         enumOptionsDeselectValue(index, checkboxesValues, enumOptions),
  //       );
  //     }
  //   };

  const _onBlur = ({ target }: FocusEvent<HTMLButtonElement>) =>
    onBlur(
      id,
      enumOptionsValueForIndex<S>(target?.value, enumOptions, emptyValue),
    );
  const _onFocus = ({ target }: FocusEvent<HTMLButtonElement>) =>
    onFocus(
      id,
      enumOptionsValueForIndex<S>(target?.value, enumOptions, emptyValue),
    );

  return (
    <>
      {!hideLabel && <p>{label}</p>}

      <div id={id}>
        {Array.isArray(enumOptions) &&
          enumOptions.map((option, index: number) => {
            const checked = enumOptionsIsSelected<S>(
              option.value,
              checkboxesValues,
            );
            const itemDisabled =
              Array.isArray(enumDisabled) &&
              enumDisabled.indexOf(option.value) !== -1;
            return (
              <Checkbox
                key={optionId(id, index)}
                id={optionId(id, index)}
                name={id}
                checked={checked}
                disabled={readonly || itemDisabled || disabled}
                autoFocus={autofocus && index === 0}
                // TODO: Fix this
                // onChange={_onChange(index)}
                onBlur={_onBlur}
                onFocus={_onFocus}
                aria-describedby={ariaDescribedByIds(id)}
              />
            );
          })}
      </div>
    </>
  );
};

export { CheckboxesWidget };
