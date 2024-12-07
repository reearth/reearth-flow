import { IChangeEvent } from "@rjsf/core";
import {
  GenericObjectType,
  RJSFSchema,
  RJSFValidationError,
} from "@rjsf/utils";
import validator from "@rjsf/validator-ajv8";
import { useState } from "react";

import { useT } from "@flow/lib/i18n";

import { ThemedForm } from "./ThemedForm";

type SchemaFormProps = {
  schema?: RJSFSchema;
  defaultFormData?: any;
  onChange?: (data: any) => void;
  onError?: (errors: any[]) => void;
  onSubmit: (data: any) => void;
};

const SchemaForm: React.FC<SchemaFormProps> = ({
  schema,
  defaultFormData,
  onChange,
  onError,
  onSubmit,
}) => {
  const t = useT();
  const [error, setError] = useState<string | null>(null);

  const handleError = (errors: RJSFValidationError[]) => {
    setError(t("Invalid data"));
    onError?.(errors);
  };

  const handleChange = (
    data: IChangeEvent<any, RJSFSchema, GenericObjectType>,
  ) => onChange?.(data.formData);

  const handleSubmit = (
    data: IChangeEvent<any, RJSFSchema, GenericObjectType>,
  ) => onSubmit(data.formData);

  return schema ? (
    <ThemedForm
      schema={schema}
      formData={defaultFormData}
      validator={validator}
      onChange={handleChange}
      onSubmit={handleSubmit}
      onError={handleError}
    />
  ) : error ? (
    <p className="text-destructive">{t("Error with the schema")}</p>
  ) : null;
};

export { SchemaForm };
