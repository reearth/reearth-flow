import {
  DescriptionFieldProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

/** The `DescriptionField` is the template to use to render the description of a field
 *
 * @param props - The `DescriptionFieldProps` for this component
 */
const DescriptionFieldTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: DescriptionFieldProps<T, S, F>,
) => {
  const { id, description } = props;
  if (description) {
    return (
      <div id={id} className="mt-1 text-xs text-muted-foreground">
        {description}
      </div>
    );
  }

  return null;
};

export { DescriptionFieldTemplate };
