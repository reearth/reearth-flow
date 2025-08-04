import {
  BaseInputTemplateProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

import {
  FieldContext,
  createFieldContext,
} from "@flow/features/Editor/components/ParamsDialog/utils/fieldUtils";

import { ColorInput } from "./BaseInputTemplate/ColorInput";
import { NumberInput } from "./BaseInputTemplate/NumberInput";
import { TextInput } from "./BaseInputTemplate/TextInput";

export type ExtendedFormContext = FormContextType & {
  onEditorOpen?: (fieldContext: FieldContext) => void;
  onAssetsOpen?: (fieldContext: FieldContext) => void;
};

/** The `BaseInputTemplate` handles all input types directly */
const BaseInputTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: BaseInputTemplateProps<T, S, F>,
) => {
  const { schema, formContext, id, name, value } = props;

  // Extract onEditorOpen from formContext
  const { onEditorOpen, onAssetsOpen } =
    (formContext as ExtendedFormContext) || {};

  // Create a field-specific onEditorOpen handler
  const handleEditorOpen = onEditorOpen
    ? () => {
        const fieldContext = createFieldContext({ id, name, value, schema });
        onEditorOpen(fieldContext);
      }
    : undefined;

  const handleAssetsOpen = onAssetsOpen
    ? () => {
        const fieldContext = createFieldContext({ id, name, value, schema });
        onAssetsOpen(fieldContext);
      }
    : undefined;

  // Handle color inputs
  if (schema.format === "color") {
    return (
      <ColorInput
        {...props}
        onEditorOpen={handleEditorOpen}
        onAssetsOpen={handleAssetsOpen}
      />
    );
  }

  // Handle number and integer inputs
  if (schema.type === "number" || schema.type === "integer") {
    return (
      <NumberInput
        {...props}
        onEditorOpen={handleEditorOpen}
        onAssetsOpen={handleAssetsOpen}
      />
    );
  }

  // Default to text input for strings and other types
  return (
    <TextInput
      {...props}
      onEditorOpen={handleEditorOpen}
      onAssetsOpen={handleAssetsOpen}
    />
  );
};

export { BaseInputTemplate };
