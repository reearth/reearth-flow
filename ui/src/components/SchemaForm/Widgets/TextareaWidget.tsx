import {
  ariaDescribedByIds,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  WidgetProps,
} from "@rjsf/utils";
import { ChangeEvent, FocusEvent } from "react";

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
  autofocus,
  readonly,
  onBlur,
  onFocus,
  onChange,
  options,
}: CustomWidgetProps<T, S, F>) => {
  const _onChange = ({ target: { value } }: ChangeEvent<HTMLTextAreaElement>) =>
    onChange(value === "" ? options.emptyValue : value);
  const _onBlur = ({ target }: FocusEvent<HTMLTextAreaElement>) =>
    onBlur(id, target?.value);
  const _onFocus = ({ target }: FocusEvent<HTMLTextAreaElement>) =>
    onFocus(id, target?.value);

  return (
    <TextArea
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
      onBlur={_onBlur}
      onFocus={_onFocus}
      aria-describedby={ariaDescribedByIds(id)}
    />
  );
};

export { TextareaWidget };
