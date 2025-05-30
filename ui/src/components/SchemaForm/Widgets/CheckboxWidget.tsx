import {
  ariaDescribedByIds,
  descriptionId,
  getTemplate,
  schemaRequiresTrueValue,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  WidgetProps,
} from "@rjsf/utils";
import { FocusEvent } from "react";

import { Checkbox } from "@flow/components";

/** The `CheckBoxWidget` is a widget for rendering boolean properties.
 *  It is typically used to represent a boolean.
 *
 * @param props - The `WidgetProps` for this component
 */
const CheckboxWidget = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: WidgetProps<T, S, F>,
) => {
  const {
    schema,
    id,
    value,
    disabled,
    readonly,
    label = "",
    hideLabel,
    autofocus,
    onChange,
    onBlur,
    onFocus,
    registry,
    options,
    uiSchema,
  } = props;
  const DescriptionFieldTemplate = getTemplate<
    "DescriptionFieldTemplate",
    T,
    S,
    F
  >("DescriptionFieldTemplate", registry, options);
  // Because an unchecked checkbox will cause html5 validation to fail, only add
  // the "required" attribute if the field value must be "true", due to the
  // "const" or "enum" keywords
  const required = schemaRequiresTrueValue<S>(schema);

  const _onChange = ({ target }: FocusEvent<HTMLButtonElement>) =>
    onChange(id, target?.value);
  const _onBlur = ({ target }: FocusEvent<HTMLButtonElement>) =>
    onBlur(id, target?.value);
  const _onFocus = ({ target }: FocusEvent<HTMLButtonElement>) =>
    onFocus(id, target?.value);
  const description = options.description ?? schema.description;

  return (
    <>
      {!hideLabel && !!description && (
        <DescriptionFieldTemplate
          id={descriptionId<T>(id)}
          description={description}
          schema={schema}
          uiSchema={uiSchema}
          registry={registry}
        />
      )}
      <div className="flex items-center gap-2">
        <Checkbox
          id={id}
          name={id}
          checked={typeof value === "undefined" ? false : Boolean(value)}
          required={required}
          disabled={readonly || disabled}
          autoFocus={autofocus}
          onChange={_onChange}
          onBlur={_onBlur}
          onFocus={_onFocus}
          onClick={() => onChange(!value)}
          aria-describedby={ariaDescribedByIds<T>(id)}
        />
        {!hideLabel && <p>{label}</p>}
      </div>
    </>
  );
};

export { CheckboxWidget };
