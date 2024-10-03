import {
  FormContextType,
  getTemplate,
  labelValue,
  RJSFSchema,
  StrictRJSFSchema,
  WidgetProps,
} from "@rjsf/utils";

const RangeWidget = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: WidgetProps<T, S, F>,
) => {
  const { value, label, hideLabel, options, registry } = props;
  const BaseInputTemplate = getTemplate<"BaseInputTemplate", T, S, F>(
    "BaseInputTemplate",
    registry,
    options,
  );
  return (
    <BaseInputTemplate
      {...props}
      extraProps={{ label: labelValue(label || undefined, hideLabel) }}>
      <span>{value}</span>
    </BaseInputTemplate>
  );
};

export { RangeWidget };
