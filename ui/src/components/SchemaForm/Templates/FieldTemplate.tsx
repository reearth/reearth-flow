import {
  FieldTemplateProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  getTemplate,
  getUiOptions,
} from "@rjsf/utils";

import { Label } from "@flow/components";

/** The `FieldTemplate` component is the template used by `SchemaField` to render any field. It renders the field
 * content, (label, description, children, errors and help) inside of a `WrapIfAdditional` component.
 *
 * @param props - The `FieldTemplateProps` for this component
 */
const FieldTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: FieldTemplateProps<T, S, F>,
) => {
  const {
    id,
    children,
    classNames,
    style,
    disabled,
    displayLabel,
    hidden,
    label,
    onDropPropertyClick,
    onKeyChange,
    readonly,
    required,
    errors,
    help,
    description,
    rawDescription,
    schema,
    uiSchema,
    registry,
  } = props;
  const uiOptions = getUiOptions<T, S, F>(uiSchema);
  const WrapIfAdditionalTemplate = getTemplate(
    "WrapIfAdditionalTemplate",
    registry,
    uiOptions,
  );

  if (hidden) {
    return <div className="hidden">{children}</div>;
  }
  return (
    <WrapIfAdditionalTemplate
      classNames={classNames}
      style={style}
      disabled={disabled}
      id={id}
      label={label}
      onDropPropertyClick={onDropPropertyClick}
      onKeyChange={onKeyChange}
      readonly={readonly}
      required={required}
      schema={schema}
      uiSchema={uiSchema}
      registry={registry}>
      <div className="my-4 w-full">
        {displayLabel && (
          <Label htmlFor={id}>
            <div className="my-1">
              {label}{" "}
              {required && <span className="text-destructive"> * </span>}
            </div>
          </Label>
        )}
        {children}
        {rawDescription && (
          <div id={id} className="mt-1 text-xs text-muted-foreground">
            {description}
          </div>
        )}
        {errors && (
          <div className="mt-1 text-xs text-destructive" role="alert">
            {errors}
          </div>
        )}
        {help && (
          <div className="mt-1 text-xs text-muted-foreground">
            {help}
          </div>
        )}
      </div>
    </WrapIfAdditionalTemplate>
  );
};

export { FieldTemplate };
