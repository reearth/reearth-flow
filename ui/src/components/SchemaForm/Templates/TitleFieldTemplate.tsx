import {
  FormContextType,
  TitleFieldProps,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

import { Label } from "@flow/components";

/** The `TitleField` is the template to use to render the title of a field
 *
 * @param props - The `TitleFieldProps` for this component
 */
const TitleFieldTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  id,
  title,
  required,
  schema,
}: TitleFieldProps<T, S, F>) => {
  const isRootTitle = schema.title === title; // Might be better way since this also includes titles in more complex schemas
  return (
    <Label id={id}>
      <div className="my-4 mb-1 flex shrink-0 flex-row items-center gap-1">
        <p className={`${isRootTitle ? "font-bold" : "font-normal"}`}>
          {title}
        </p>
        {required && <p className="h-2 font-thin text-destructive">*</p>}
      </div>
      <div className="border-b" />
    </Label>
  );
};

export { TitleFieldTemplate };
