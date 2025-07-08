import {
  BaseInputTemplateProps,
  getInputProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

import { ColorInput } from "./BaseInputTemplate/ColorInput";
import { DefaultTextArea } from "./BaseInputTemplate/DefaultTextArea";
import { NumberInput } from "./BaseInputTemplate/NumberInput";
import { TextInput } from "./BaseInputTemplate/TextInput";

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
    errorSchema,
    formContext,
    registry,
    InputLabelProps,
    ...textFieldProps
  } = props;

  const inputProps = getInputProps<T, S, F>(schema, type, options);
  const { step, min, max, ...rest } = inputProps;
  const otherProps = {
    inputProps: {
      step,
      min,
      max,
    },
    ...rest,
  };

  if (schema.format === "color") {
    return <ColorInput props={props} inputProps={otherProps} />;
  }

  if (schema.format === "text") {
    return <TextInput props={props} inputProps={otherProps} />;
  }

  // Handle number types (integer, number) or explicit number format
  if (
    schema.type === "number" ||
    schema.type === "integer" ||
    type === "number"
  ) {
    return <NumberInput props={props} inputProps={otherProps} />;
  }

  return (
    <DefaultTextArea
      props={props}
      inputProps={otherProps}
      textFieldProps={textFieldProps}
    />
  );
};

export { BaseInputTemplate };
