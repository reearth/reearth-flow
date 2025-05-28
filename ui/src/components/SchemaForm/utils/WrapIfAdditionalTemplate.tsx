import {
  ADDITIONAL_PROPERTY_FLAG,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  WrapIfAdditionalTemplateProps,
} from "@rjsf/utils";
import { FocusEvent } from "react";

import { Input } from "@flow/components/Input";
import { cn } from "@flow/lib/utils";

/** The `WrapIfAdditional` component is used by the `FieldTemplate` to rename, or remove properties that are
 * part of an `additionalProperties` part of a schema.
 *
 * @param props - The `WrapIfAdditionalProps` for this component
 */
const WrapIfAdditionalTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: WrapIfAdditionalTemplateProps<T, S, F>,
) => {
  const {
    children,
    classNames,
    disabled,
    id,
    label,
    onDropPropertyClick,
    onKeyChange,
    readonly,
    required,
    schema,
    uiSchema,
    registry,
  } = props;
  const { templates } = registry;
  // Button templates are not overridden in the uiSchema
  const { RemoveButton } = templates.ButtonTemplates;
  const additional = ADDITIONAL_PROPERTY_FLAG in schema;

  if (!additional) {
    return <div className={classNames}>{children}</div>;
  }

  const handleBlur = ({ target }: FocusEvent<HTMLInputElement>) =>
    onKeyChange(target && target.value);

  return (
    <div key={`${id}-key`} className={cn("align-middle gap-4", classNames)}>
      <div>
        <Input
          required={required}
          defaultValue={label}
          disabled={readonly || disabled}
          id={`${id}-key`}
          name={`${id}-key`}
          onBlur={!readonly ? handleBlur : undefined}
          type="text"
        />
      </div>
      <div>{children}</div>
      <div>
        <RemoveButton
          iconType="default"
          disabled={readonly || disabled}
          onClick={onDropPropertyClick(label)}
          uiSchema={uiSchema}
          registry={registry}
        />
      </div>
    </div>
  );
};

export { WrapIfAdditionalTemplate };
