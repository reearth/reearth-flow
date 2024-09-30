import {
  FieldHelpProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  helpId,
} from "@rjsf/utils";

/** The `FieldHelpTemplate` component renders any help desired for a field
 *
 * @param props - The `FieldHelpProps` to be rendered
 */
const FieldHelpTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: FieldHelpProps<T, S, F>,
) => {
  const { idSchema, help, hasErrors } = props;
  if (!help) {
    return null;
  }
  const id = helpId<T>(idSchema);
  return (
    <div className={hasErrors ? "text-destructive" : "text-muted"} id={id}>
      {help}
    </div>
  );
};

export { FieldHelpTemplate };
