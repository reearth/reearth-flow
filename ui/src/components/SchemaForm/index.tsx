import { RJSFSchema } from "@rjsf/utils";
import validator from "@rjsf/validator-ajv8";

import { useT } from "@flow/lib/i18n";

import { ThemedForm } from "./ThemedForm";

type SchemaFormProps = {
  schema?: RJSFSchema;
};

const SchemaForm: React.FC<SchemaFormProps> = ({ schema }) => {
  // TODO: Temporary, will accept these events as props
  const log = (type: any) => console.log.bind(console, type);

  const t = useT();

  return schema ? (
    <ThemedForm
      schema={schema}
      validator={validator}
      onChange={log("changed")}
      onSubmit={log("submitted")}
      onError={log("errors")}
    />
  ) : (
    <div className="text-destructive ">{t("Schema not found")}</div>
  );
};

export { SchemaForm };
