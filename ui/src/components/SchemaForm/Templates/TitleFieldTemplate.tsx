import {
  FormContextType,
  TitleFieldProps,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

/** The `TitleField` is the template to use to render the title of a field
 *
 * @param props - The `TitleFieldProps` for this component
 */
const TitleFieldTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = any,
>({
  id,
  title,
  required,
}: TitleFieldProps<T, S, F>) => {
  return (
    <div id={id} className="my-1">
      <div className="text-xl">
        {title} {required && <div className="text-destructive"> * </div>}
      </div>
      <div className="border" />
    </div>
  );
};

export { TitleFieldTemplate };
