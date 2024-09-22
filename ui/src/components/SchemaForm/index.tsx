import { RJSFSchema } from "@rjsf/utils";
import validator from "@rjsf/validator-ajv8";

import { Form } from "./Theme";

const SchemaForm = () => {
  const schema: RJSFSchema = {
    title: "Todo",
    type: "object",
    required: ["title"],
    properties: {
      title: { type: "string", title: "Title", default: "A new task" },
      done: {
        type: "boolean",
        title: "Done?",
        default: false,
        description: "Sample description",
      },
    },
  };

  const log = (type: any) => console.log.bind(console, type);

  return (
    <Form
      schema={schema}
      validator={validator}
      onChange={log("changed")}
      onSubmit={log("submitted")}
      onError={log("errors")}
    />
  );
};

export { SchemaForm };
