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
  onPythonEditorOpen?: (fieldContext: FieldContext) => void;
  onAssetsOpen?: (fieldContext: FieldContext) => void;
  originalSchema?: any;
  actionName?: string;
};

/** The `BaseInputTemplate` handles all input types directly */
const BaseInputTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: BaseInputTemplateProps<T, S, F>,
) => {
  const { schema, registry, id, name, value, uiSchema } = props;

  const formContext = registry.formContext as ExtendedFormContext;

  const {
    onEditorOpen,
    onPythonEditorOpen,
    onAssetsOpen,
    originalSchema,
    actionName,
  } = formContext || {};

  // Check if this field is marked as an Expr type in the UI schema
  let isExprField = uiSchema?.["ui:exprType"] === "rhai";
  let isPythonField = uiSchema?.["ui:exprType"] === "python";

  // Fallback: detect expression types from schema or originalSchema (for dynamic array items)
  if (!isExprField && !isPythonField) {
    // Dynamically check if ANY definition in originalSchema has this field name with Expr support
    let hasExprSupport = false;
    if (originalSchema?.definitions) {
      hasExprSupport = Object.values(originalSchema.definitions).some(
        (def: any) =>
          def?.properties?.[name]?.$ref === "#/definitions/Expr" ||
          def?.properties?.[name]?.allOf?.some(
            (item: any) => item.$ref === "#/definitions/Expr",
          ),
      );
    }

    if (hasExprSupport) {
      // Only treat as Python script if it's specifically PythonScriptProcessor and the field is 'script'
      isPythonField =
        actionName === "PythonScriptProcessor" && name === "script";
      isExprField = !isPythonField;
    }
  }

  // Create field-specific editor handlers
  const handleEditorOpen =
    onEditorOpen && isExprField
      ? () => {
          const fieldContext = createFieldContext({ id, name, value, schema });
          onEditorOpen(fieldContext);
        }
      : undefined;

  const handlePythonEditorOpen =
    onPythonEditorOpen && isPythonField
      ? () => {
          const fieldContext = createFieldContext({ id, name, value, schema });
          onPythonEditorOpen(fieldContext);
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

  // Handle number and integer inputs (including arrays like ["integer", "null"])
  const isNumberType =
    schema.type === "number" ||
    schema.type === "integer" ||
    (Array.isArray(schema.type) &&
      (schema.type.includes("number") || schema.type.includes("integer")));

  if (isNumberType) {
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
      onPythonEditorOpen={handlePythonEditorOpen}
      onAssetsOpen={handleAssetsOpen}
    />
  );
};

export { BaseInputTemplate };
