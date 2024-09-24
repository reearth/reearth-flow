import { RJSFSchema } from "@rjsf/utils";
import validator from "@rjsf/validator-ajv8";

import { ThemedForm } from "./ThemedForm";

type SchemaFormProps = {
  schema: RJSFSchema;
};

const SchemaForm: React.FC<SchemaFormProps> = ({ schema }) => {
  // TODO: Temporary, will accept these events as props
  const log = (type: any) => console.log.bind(console, type);

  return (
    <ThemedForm
      schema={schema}
      validator={validator}
      onChange={log("changed")}
      onSubmit={log("submitted")}
      onError={log("errors")}
    />
  );
};

export { SchemaForm };
