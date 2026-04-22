import { IChangeEvent } from "@rjsf/core";
import {
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
};

// Function to recursively scan schema for Expr types and build UI schema
const buildExprUiSchema = (
  schemaObj: any,
  actionName?: string,
  path = "",
): any => {
  if (!schemaObj || typeof schemaObj !== "object") return {};
  const uiSchema: any = {};

  if (schemaObj.format === "wysiwyg") {
    return { "ui:widget": "WysiwygWidget" };
  }

  // Determine if this is a Python script field or regular Rhai expression

  const isExprType =
    schemaObj.$ref === "#/definitions/Expr" ||
    schemaObj.allOf?.some((item: any) => item.$ref === "#/definitions/Expr") ||
    schemaObj.anyOf?.some((item: any) => item.$ref === "#/definitions/Expr");

  // Check if this schema references any definition that contains expressions
  let referencesExprDefinition = false;
  if (schemaObj.$ref) {
    const refName = schemaObj.$ref.replace("#/definitions/", "");
    const referencedDef = schemaObj.definitions?.[refName];
    if (referencedDef?.properties) {
      referencesExprDefinition = Object.values(referencedDef.properties).some(
        (prop: any) =>
          prop?.$ref === "#/definitions/Expr" ||
          prop?.allOf?.some(
            (item: any) => item.$ref === "#/definitions/Expr",
          ) ||
          prop?.anyOf?.some((item: any) => item.$ref === "#/definitions/Expr"),
      );
    }
  }

  // Handle schemas that define ANY type with expression fields - apply UI schema dynamically
  let hasExprDefinitions = false;
  const exprFieldUiSchema: any = {};
  if (schemaObj.definitions) {
    Object.entries(schemaObj.definitions).forEach(
      ([_defName, def]: [string, any]) => {
        if (def?.properties) {
          Object.entries(def.properties).forEach(
            ([propName, prop]: [string, any]) => {
              const propHasExpr =
                prop?.$ref === "#/definitions/Expr" ||
                prop?.allOf?.some(
                  (item: any) => item.$ref === "#/definitions/Expr",
                ) ||
                prop?.anyOf?.some(
                  (item: any) => item.$ref === "#/definitions/Expr",
                );

              if (propHasExpr) {
                hasExprDefinitions = true;
                exprFieldUiSchema[propName] = { "ui:exprType": "rhai" };
              }
            },
          );
        }
      },
    );
  }

  if (isExprType || referencesExprDefinition) {
    const fieldName = path.split(".").pop() || "";

    // Only treat as Python script if it's specifically PythonScriptProcessor and the field is 'script'
    const isPythonScript =
      actionName === "PythonScriptProcessor" && fieldName === "script";

    return {
      "ui:exprType": isPythonScript ? "python" : "rhai",
    };
  }

  // Return dynamic UI schema for expression fields found in definitions
  if (hasExprDefinitions) {
    return {
      ...exprFieldUiSchema,
    };
  }

  // Recursively check properties
  if (schemaObj.properties) {
    for (const [key, value] of Object.entries(schemaObj.properties)) {
      const childPath = path ? `${path}.${key}` : key;
      const childUiSchema = buildExprUiSchema(value, actionName, childPath);
      if (Object.keys(childUiSchema).length > 0) {
        uiSchema[key] = childUiSchema;
      }
    }
  }

  // Also recursively check allOf, oneOf, anyOf structures
  if (schemaObj.allOf) {
    for (const subSchema of schemaObj.allOf) {
      const childUiSchema = buildExprUiSchema(subSchema, actionName, path);
      Object.assign(uiSchema, childUiSchema);
    }
  }

  if (schemaObj.oneOf) {
    for (const subSchema of schemaObj.oneOf) {
      const childUiSchema = buildExprUiSchema(subSchema, actionName, path);
      Object.assign(uiSchema, childUiSchema);
    }
  }

  if (schemaObj.anyOf) {
    for (const subSchema of schemaObj.anyOf) {
      const childUiSchema = buildExprUiSchema(subSchema, actionName, path);
      Object.assign(uiSchema, childUiSchema);
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
        const validationResult = validator.validateFormData(
          defaultFormData,
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
