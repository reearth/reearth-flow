import {
  BaseInputTemplateProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

import { ColorInput } from "./BaseInputTemplate/ColorInput";
import { NumberInput } from "./BaseInputTemplate/NumberInput";
import { TextInput } from "./BaseInputTemplate/TextInput";

/** The `BaseInputTemplate` handles all input types directly */
const BaseInputTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: BaseInputTemplateProps<T, S, F>,
) => {
  const { schema } = props;

  // Handle color inputs
  if (schema.format === "color") {
    return <ColorInput {...props} />;
  }

  // Handle number and integer inputs
  if (schema.type === "number" || schema.type === "integer") {
    return <NumberInput {...props} />;
  }

  // Default to text input for strings and other types
  return <TextInput {...props} />;
};

export { BaseInputTemplate };