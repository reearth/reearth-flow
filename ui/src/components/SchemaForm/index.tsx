import { IChangeEvent } from "@rjsf/core";
import {
  GenericObjectType,
  RJSFSchema,
  RJSFValidationError,
} from "@rjsf/utils";
import validator from "@rjsf/validator-ajv8";
import { useState, useEffect } from "react";

import { FieldContext } from "@flow/features/Editor/components/ParamsDialog/utils/fieldUtils";
import { useT } from "@flow/lib/i18n";

import { SchemaFormErrorBoundary } from "./components/SchemaFormErrorBoundary";
import { ThemedForm } from "./ThemedForm";

type SchemaFormProps = {
  readonly?: boolean;
  schema?: RJSFSchema;
  originalSchema?: any; // Original schema before patching, used for UI schema generation
  actionName?: string; // Action name to help identify field types
  defaultFormData?: any;
  onChange: (data: any) => void;
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

  // Add description to UI schema if available
  if (schemaObj.description) {
    uiSchema["ui:description"] = schemaObj.description;
  }

  // if (schemaObj.definitions.Operation.properties) {
  //   uiSchema["ui:description"] = schemaObj.description;
  // }

  // Determine if this is a Python script field or regular Rhai expression
  const isExprType =
    schemaObj.$ref === "#/definitions/Expr" ||
    schemaObj.allOf?.some((item: any) => item.$ref === "#/definitions/Expr");

  if (isExprType) {
    const fieldName = path.split(".").pop() || "";

    // Only treat as Python script if it's specifically PythonScriptProcessor and the field is 'script'
    const isPythonScript =
      actionName === "PythonScriptProcessor" && fieldName === "script";

    return {
      "ui:exprType": isPythonScript ? "python" : "rhai",
      ...(schemaObj.description
        ? { "ui:description": schemaObj.description }
        : {}),
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
  schema,
  originalSchema,
  actionName,
  defaultFormData,
  onChange,
  onError,
  onValidationChange,
  onEditorOpen,
  onPythonEditorOpen,
}) => {
  const t = useT();
  const [error, setError] = useState<string | null>(null);

  const handleError = (errors: RJSFValidationError[]) => {
    const hasValidationErrors = errors.length > 0;
    setError(hasValidationErrors ? t("Invalid data") : null);
    onValidationChange?.(!hasValidationErrors);
    onError?.(errors);
  };

  const handleChange = (
    data: IChangeEvent<any, RJSFSchema, GenericObjectType>,
  ) => {
    const hasValidationErrors = data.errors && data.errors.length > 0;

    if (hasValidationErrors) {
      setError(t("Invalid data"));
      onValidationChange?.(false);
    } else {
      setError(null);
      onValidationChange?.(true);
    }

    onChange(data.formData);
  };

  // Validate initial data on mount
  useEffect(() => {
    if (schema && defaultFormData) {
      try {
        const validationResult = validator.validateFormData(
          defaultFormData,
          schema,
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
  }, [schema, defaultFormData, onValidationChange, t]);

  // Generate UI schema to mark Expr fields from original schema (before patching)

  const exprUiSchema = originalSchema
    ? buildExprUiSchema(originalSchema, actionName)
    : {};

  const finalUiSchema = {
    ...exprUiSchema,

    "ui:submitButtonOptions": { norender: true },
  };
  console.log("ORIGINAL SCHEMA", finalUiSchema);
  return schema ? (
    <SchemaFormErrorBoundary>
      <ThemedForm
        className="flex-1 overflow-scroll"
        schema={schema}
        readonly={readonly}
        formData={defaultFormData}
        validator={validator}
        uiSchema={finalUiSchema}
        formContext={{ onEditorOpen, onPythonEditorOpen }}
        onChange={handleChange}
        onError={handleError}
      />
    </SchemaFormErrorBoundary>
  ) : error ? (
    <p className="text-destructive">{t("Error with the schema")}</p>
  ) : null;
};

export { SchemaForm };
