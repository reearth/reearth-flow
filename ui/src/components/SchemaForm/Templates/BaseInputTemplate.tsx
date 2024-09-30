import {
  ariaDescribedByIds,
  BaseInputTemplateProps,
  examplesId,
  getInputProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";
import { ChangeEvent, FocusEvent } from "react";

import { Input } from "@flow/components";

/** The `BaseInputTemplate` is the template to use to render the basic `<input>` component for the `core` theme.
 * It is used as the template for rendering many of the <input> based widgets that differ by `type` and callbacks only.
 * It can be customized/overridden for other themes or individual implementations as needed.
 *
 * @param props - The `WidgetProps` for this template
 */
const BaseInputTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: BaseInputTemplateProps<T, S, F>,
) => {
  const {
    id,
    name, // remove this from textFieldProps
    placeholder,
    required,
    readonly,
    disabled,
    type,
    label,
    hideLabel,
    hideError,
    value,
    onChange,
    onChangeOverride,
    onBlur,
    onFocus,
    autofocus,
    options,
    schema,
    uiSchema,
    rawErrors = [],
    errorSchema,
    formContext,
    registry,
    InputLabelProps,
    ...textFieldProps
  } = props;
  const inputProps = getInputProps<T, S, F>(schema, type, options);
  // Now we need to pull out the step, min, max into an inner `inputProps` for material-ui
  const { step, min, max, ...rest } = inputProps;
  const otherProps = {
    inputProps: {
      step,
      min,
      max,
      ...(schema.examples ? { list: examplesId<T>(id) } : undefined),
    },
    ...rest,
  };
  const _onChange = ({ target: { value } }: ChangeEvent<HTMLInputElement>) =>
    onChange(value === "" ? options.emptyValue : value);
  const _onBlur = ({ target }: FocusEvent<HTMLInputElement>) =>
    onBlur(id, target && target.value);
  const _onFocus = ({ target }: FocusEvent<HTMLInputElement>) =>
    onFocus(id, target && target.value);

  return (
    <>
      <Input
        id={id}
        name={id}
        placeholder={placeholder}
        autoFocus={autofocus}
        required={required}
        disabled={disabled || readonly}
        {...otherProps}
        value={value || value === 0 ? value : ""}
        onChange={onChangeOverride || _onChange}
        onBlur={_onBlur}
        onFocus={_onFocus}
        {...textFieldProps}
        aria-describedby={ariaDescribedByIds<T>(id, !!schema.examples)}
      />
      {Array.isArray(schema.examples) && (
        <datalist id={examplesId<T>(id)}>
          {(schema.examples as string[])
            .concat(
              schema.default && !schema.examples.includes(schema.default)
                ? ([schema.default] as string[])
                : [],
            )
            .map((example: any) => {
              return <option key={example} value={example} />;
            })}
        </datalist>
      )}
      {rawErrors.length > 0 &&
        rawErrors.map((e, i) => (
          <p key={i} className="text-xs text-destructive">
            {e}
          </p>
        ))}
    </>
  );
};

export { BaseInputTemplate };
