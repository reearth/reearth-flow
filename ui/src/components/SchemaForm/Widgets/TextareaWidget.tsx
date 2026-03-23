import {
  ariaDescribedByIds,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  WidgetProps,
} from "@rjsf/utils";
import { ChangeEvent, useCallback } from "react";

import { TextArea } from "@flow/components";

type CustomWidgetProps<
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
> = WidgetProps<T, S, F> & {
  options: any;
};

const TextareaWidget = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  id,
  placeholder,
  value,
  required,
  disabled,
  registry,
  autofocus,
  readonly,
  onBlur,
  onFocus,
  onChange,
  options,
}: CustomWidgetProps<T, S, F>) => {
  const formContext = registry?.formContext;
  const { fieldFocusMap, onFieldFocus } = formContext ?? {};
  const focusedUsers = fieldFocusMap?.[id] ?? [];
  const _onChange = ({ target: { value } }: ChangeEvent<HTMLTextAreaElement>) =>
    onChange(value === "" ? options.emptyValue : value);

  const handleBlur = useCallback(() => {
    onBlur(id, value);

    onFieldFocus?.(null);
  }, [onBlur, id, onFieldFocus, value]);
  const handleFocus = useCallback(() => {
    onFocus(id, value);
    onFieldFocus?.(id);
  }, [onFocus, id, onFieldFocus, value]);

  return (
    <TextArea
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
      id={id}
      name={id}
      placeholder={placeholder}
      disabled={disabled}
      readOnly={readonly}
      value={value}
      required={required}
      autoFocus={autofocus}
      rows={options.rows || 5}
      onChange={_onChange}
      onBlur={handleBlur}
      onFocus={handleFocus}
      aria-describedby={ariaDescribedByIds(id)}
    />
  );
};

export { TextareaWidget };
