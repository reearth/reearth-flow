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
  const { schema, formContext } = props;

  // Extract onEditorOpen from formContext
  const { onEditorOpen } = (formContext as any) || {};

  // Handle color inputs
  if (schema.format === "color") {
    return <ColorInput {...props} onEditorOpen={onEditorOpen} />;
  }

  // Handle number and integer inputs
  if (schema.type === "number" || schema.type === "integer") {
    return <NumberInput {...props} onEditorOpen={onEditorOpen} />;
  }

  // Default to text input for strings and other types
  return <TextInput {...props} onEditorOpen={onEditorOpen} />;
};

export { BaseInputTemplate };