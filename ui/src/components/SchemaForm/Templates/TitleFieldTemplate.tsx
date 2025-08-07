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
}: TitleFieldProps<T, S, F>) => {
  return (
    <Label id={id} className="mt-2 mb-4 first-letter:uppercase">
      <div className="flex flex-row items-center gap-1">
        <p className="font-light">{title}</p>
        {required && <p className="font-thin text-destructive">*</p>}
      </div>
    </Label>
  );
};

export { TitleFieldTemplate };
