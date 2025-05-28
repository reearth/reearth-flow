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
  onChange: (data: any) => void;
  onError?: (errors: any[]) => void;
  disableEditing?: boolean; // Optional prop to disable editing
};

const SchemaForm: React.FC<SchemaFormProps> = ({
  schema,
  defaultFormData,
  onChange,
  onError,
  disableEditing = false,
}) => {
  const t = useT();
  const [error, setError] = useState<string | null>(null);

  const handleError = (errors: RJSFValidationError[]) => {
    setError(t("Invalid data"));
    onError?.(errors);
  };

  const handleChange = (
    data: IChangeEvent<any, RJSFSchema, GenericObjectType>,
  ) => onChange(data.formData);

  return schema ? (
    <ThemedForm
      className="flex-1 overflow-scroll"
      schema={schema}
      disabled={disableEditing}
      formData={defaultFormData}
      validator={validator}
      uiSchema={{ "ui:submitButtonOptions": { norender: true } }} // We handle submissions outside of this component
      onChange={handleChange}
      onError={handleError}
    />
  ) : error ? (
    <p className="text-destructive">{t("Error with the schema")}</p>
  ) : null;
};

export { SchemaForm };
