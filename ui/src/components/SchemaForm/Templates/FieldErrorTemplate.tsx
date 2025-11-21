import {
  FieldErrorProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  errorId,
} from "@rjsf/utils";

/** The `FieldErrorTemplate` component renders the errors local to the particular field
 *
 * @param props - The `FieldErrorProps` for the errors being rendered
 */
const FieldErrorTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: FieldErrorProps<T, S, F>,
) => {
  const { errors = [], fieldPathId } = props;
  if (errors.length === 0) {
    return null;
  }
  const id = errorId(fieldPathId);

  return (
    <div id={id}>
      {errors.map((error, i) => {
        return (
          <div key={i}>
            <small className="text-destructive">{error}</small>
          </div>
        );
      })}
    </div>
  );
};

export { FieldErrorTemplate };
