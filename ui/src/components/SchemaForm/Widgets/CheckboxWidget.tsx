import {
  ariaDescribedByIds,
  schemaRequiresTrueValue,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  WidgetProps,
} from "@rjsf/utils";
import { FocusEvent, useCallback } from "react";

import { Checkbox } from "@flow/components";

/** The `CheckBoxWidget` is a widget for rendering boolean properties.
 *  It is typically used to represent a boolean.
 *
 * @param props - The `WidgetProps` for this component
 */
const CheckboxWidget = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: WidgetProps<T, S, F>,
) => {
  const {
    schema,
    id,
    value,
    disabled,
    readonly,
    label = "",
    registry,
    hideLabel,
    autofocus,
    onChange,
    onBlur,
    onFocus,
  } = props;

  // Because an unchecked checkbox will cause html5 validation to fail, only add
  // the "required" attribute if the field value must be "true", due to the
  // "const" or "enum" keywords
  const required = schemaRequiresTrueValue<S>(schema);
  const formContext = registry?.formContext;
  const { fieldFocusMap, onFieldFocus } = formContext ?? {};
  const focusedUsers = fieldFocusMap?.[id] ?? [];
  const _onChange = ({ target }: FocusEvent<HTMLButtonElement>) =>
    onChange(id, target?.value);

  const handleBlur = useCallback(
    ({ target }: FocusEvent<HTMLButtonElement>) => {
      onBlur?.(id, target.value);
      onFieldFocus?.(null);
    },
    [onBlur, onFieldFocus, id],
  );

  const handleFocus = useCallback(
    ({ target }: FocusEvent<HTMLButtonElement>) => {
      onFocus?.(id, target.value);
      onFieldFocus?.(id);
    },
    [onFocus, onFieldFocus, id],
  );

  return (
    <div className="flex items-center gap-2 py-2">
      <Checkbox
        id={id}
        name={id}
        style={{
          border:
            Array.isArray(focusedUsers) && focusedUsers.length > 0
              ? "2px solid"
              : undefined,
          borderColor:
            Array.isArray(focusedUsers) && focusedUsers.length > 0
              ? focusedUsers.map((user) => user.color).join(",")
              : undefined,
          borderRadius:
            Array.isArray(focusedUsers) && focusedUsers.length > 0
              ? "4px"
              : undefined,
        }}
        checked={typeof value === "undefined" ? false : Boolean(value)}
        required={required}
        disabled={readonly || disabled}
        autoFocus={autofocus}
        onChange={_onChange}
        onBlur={handleBlur}
        onFocus={handleFocus}
        onClick={() => onChange(!value)}
        aria-describedby={ariaDescribedByIds(id)}
      />
      {!hideLabel && <p className="text-xs">{label}</p>}
    </div>
  );
};

export { CheckboxWidget };
