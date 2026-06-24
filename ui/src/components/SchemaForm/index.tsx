import { IChangeEvent } from "@rjsf/core";
import {
  createSchemaUtils,
  GenericObjectType,
  RJSFSchema,
  RJSFValidationError,
} from "@rjsf/utils";
import validator from "@rjsf/validator-ajv8";
import { JSONSchema7Definition } from "json-schema";
import { useState, useEffect, useMemo } from "react";

import { patchAnyOfAndOneOfType } from "@flow/components/SchemaForm/patchSchemaTypes";
import { FieldContext } from "@flow/features/Editor/components/ParamsDialog/utils/fieldUtils";
import { useT } from "@flow/lib/i18n";
import { AwarenessUser } from "@flow/types";

import { SchemaFormErrorBoundary } from "./components/SchemaFormErrorBoundary";
import { ThemedForm } from "./ThemedForm";

type SchemaFormProps = {
  readonly?: boolean;
  schema?: any; // Original schema before patching, used for UI schema generation
  actionName?: string; // Action name to help identify field types
  defaultFormData?: any;
  fieldFocusMap?: Record<string, AwarenessUser[]>;
  onFieldFocus?: (fieldId: string | null) => void;
  onChange: (data: any, changedFieldId?: string) => void;
  onError?: (errors: RJSFValidationError[]) => void;
  onValidationChange?: (isValid: boolean) => void;
  onEditorOpen?: (fieldContext: FieldContext) => void;
  onPythonEditorOpen?: (fieldContext: FieldContext) => void;
  onFlowExprEditorOpen?: (fieldContext: FieldContext) => void;
};

// Function to recursively scan schema for Expr/Code types and build UI schema.
// rootDefinitions is threaded through recursion so $ref can be resolved at any depth.
const buildExprUiSchema = (
  schemaObj: any,
  actionName?: string,
  path = "",
  rootDefinitions?: Record<string, any>,
): any => {
  if (!schemaObj || typeof schemaObj !== "object") return {};

  // Capture root definitions on the first call; reuse them in every recursive call.
  const defs: Record<string, any> =
    rootDefinitions ?? schemaObj.definitions ?? {};

  if (schemaObj.format === "wysiwyg") {
    return { "ui:widget": "WysiwygWidget" };
  }

  const isCodeType =
    schemaObj.format === "code" ||
    schemaObj.allOf?.some((item: any) => item.$ref === "#/definitions/Code") ||
    schemaObj.anyOf?.some((item: any) => item.$ref === "#/definitions/Code");

  if (isCodeType) {
    return { "ui:field": "FlowExprField" };
  }

  const isExprType =
    schemaObj.$ref === "#/definitions/Expr" ||
    schemaObj.allOf?.some((item: any) => item.$ref === "#/definitions/Expr") ||
    schemaObj.anyOf?.some((item: any) => item.$ref === "#/definitions/Expr");

  if (isExprType) {
    const fieldName = path.split(".").pop() || "";
    const isPythonScript =
      actionName === "PythonScriptProcessor" && fieldName === "script";
    return { "ui:exprType": isPythonScript ? "python" : "flowExpr" };
  }

  // Resolve a plain $ref to its definition and recurse into it.
  if (schemaObj.$ref) {
    const refName = (schemaObj.$ref as string).replace("#/definitions/", "");
    const refDef = defs[refName];
    if (refDef) {
      return buildExprUiSchema(refDef, actionName, path, defs);
    }
    return {};
  }

  const uiSchema: any = {};

  // Recurse into object properties.
  if (schemaObj.properties) {
    for (const [key, value] of Object.entries(schemaObj.properties)) {
      const childPath = path ? `${path}.${key}` : key;
      const childUiSchema = buildExprUiSchema(
        value,
        actionName,
        childPath,
        defs,
      );
      if (Object.keys(childUiSchema).length > 0) {
        uiSchema[key] = childUiSchema;
      }
    }
  }

  // Recurse into array items so nested Code/Expr fields inside arrays are found.
  if (schemaObj.items) {
    const itemsUiSchema = buildExprUiSchema(
      schemaObj.items,
      actionName,
      path,
      defs,
    );
    if (Object.keys(itemsUiSchema).length > 0) {
      uiSchema.items = itemsUiSchema;
    }
  }

  // Recurse into allOf / oneOf / anyOf.
  for (const keyword of ["allOf", "oneOf", "anyOf"] as const) {
    if (schemaObj[keyword]) {
      for (const subSchema of schemaObj[keyword]) {
        const childUiSchema = buildExprUiSchema(
          subSchema,
          actionName,
          path,
          defs,
        );
        Object.assign(uiSchema, childUiSchema);
      }
    }
  }

  return uiSchema;
};

const SchemaForm: React.FC<SchemaFormProps> = ({
  readonly,
  schema: originalSchema,
  actionName,
  defaultFormData,
  fieldFocusMap,
  onFieldFocus,
  onChange,
  onError,
  onValidationChange,
  onEditorOpen,
  onPythonEditorOpen,
  onFlowExprEditorOpen,
}) => {
  const t = useT();
  const [error, setError] = useState<string | null>(null);

  // This is a patch for the `anyOf` type in JSON Schema.
  const patchedSchema = useMemo<RJSFSchema | undefined>(
    () =>
      originalSchema
        ? patchAnyOfAndOneOfType(originalSchema as JSONSchema7Definition)
        : undefined,
    [originalSchema],
  );

  const handleError = (errors: RJSFValidationError[]) => {
    const hasValidationErrors = errors.length > 0;
    setError(hasValidationErrors ? t("Invalid data") : null);
    onValidationChange?.(!hasValidationErrors);
    onError?.(errors);
  };

  const handleChange = (
    data: IChangeEvent<any, RJSFSchema, GenericObjectType>,
    changedFieldId?: string,
  ) => {
    const hasValidationErrors = data.errors && data.errors.length > 0;

    if (hasValidationErrors) {
      setError(t("Invalid data"));
      onValidationChange?.(false);
    } else {
      setError(null);
      onValidationChange?.(true);
    }

    onChange(data.formData, changedFieldId);
  };

  // Validate initial data on mount
  useEffect(() => {
    if (patchedSchema && defaultFormData) {
      try {
        const schemaUtils = createSchemaUtils(validator, patchedSchema);
        const formDataWithDefaults = schemaUtils.getDefaultFormState(
          patchedSchema,
          defaultFormData,
        );
        const validationResult = validator.validateFormData(
          formDataWithDefaults,
          patchedSchema,
        );
        const isValid =
          !validationResult.errors || validationResult.errors.length === 0;
        onValidationChange?.(isValid);

        if (!isValid && validationResult.errors) {
          setError(t("Invalid data"));
        }
      } catch (err) {
        console.error("Validation error:", err);
        onValidationChange?.(false);
      }
    }
  }, [patchedSchema, defaultFormData, onValidationChange, t]);

  // Generate UI schema to mark Expr fields from original schema (before patching)

  const exprUiSchema = originalSchema
    ? buildExprUiSchema(originalSchema, actionName)
    : {};

  const finalUiSchema = {
    ...exprUiSchema,

    "ui:submitButtonOptions": { norender: true },
  };

  return patchedSchema ? (
    <SchemaFormErrorBoundary>
      <ThemedForm
        className="flex-1 overflow-scroll"
        schema={patchedSchema}
        readonly={readonly}
        formData={defaultFormData}
        validator={validator}
        uiSchema={finalUiSchema}
        formContext={{
          onEditorOpen,
          onPythonEditorOpen,
          onFlowExprEditorOpen,
          originalSchema,
          patchedSchema,
          actionName,
          fieldFocusMap,
          onFieldFocus,
        }}
        onChange={handleChange}
        onError={handleError}
      />
    </SchemaFormErrorBoundary>
  ) : error ? (
    <p className="text-destructive">{t("Error with the schema")}</p>
  ) : null;
};

export { SchemaForm };
