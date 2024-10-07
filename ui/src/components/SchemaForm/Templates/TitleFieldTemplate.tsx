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
    <Label id={id} className="my-1">
      <div className="text-xl">
        {title} {required && <div className="text-destructive"> * </div>}
      </div>
      <div className="border" />
    </Label>
  );
};

export { TitleFieldTemplate };
