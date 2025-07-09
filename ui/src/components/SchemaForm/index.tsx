import { IChangeEvent } from "@rjsf/core";
import {
  GenericObjectType,
  RJSFSchema,
  RJSFValidationError,
} from "@rjsf/utils";
import validator from "@rjsf/validator-ajv8";
import { useState, useEffect } from "react";

import { useT } from "@flow/lib/i18n";

import { SchemaFormErrorBoundary } from "./components/SchemaFormErrorBoundary";
import { ThemedForm } from "./ThemedForm";

type SchemaFormProps = {
  readonly?: boolean;
  schema?: RJSFSchema;
  defaultFormData?: any;
  onChange: (data: any) => void;
  onError?: (errors: RJSFValidationError[]) => void;
  onValidationChange?: (isValid: boolean) => void;
  onEditorOpen?: () => void;
};

const SchemaForm: React.FC<SchemaFormProps> = ({
  readonly,
  schema,
  defaultFormData,
  onChange,
  onError,
  onValidationChange,
  onEditorOpen,
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

  return schema ? (
    <SchemaFormErrorBoundary>
      <ThemedForm
        className="flex-1 overflow-scroll"
        schema={schema}
        readonly={readonly}
        formData={defaultFormData}
        validator={validator}
        uiSchema={{ "ui:submitButtonOptions": { norender: true } }} // We handle submissions outside of this component
        formContext={{ onEditorOpen }}
        onChange={handleChange}
        onError={handleError}
      />
    </SchemaFormErrorBoundary>
  ) : error ? (
    <p className="text-destructive">{t("Error with the schema")}</p>
  ) : null;
};

export { SchemaForm };
