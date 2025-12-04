import {
  ArrayFieldTemplateProps,
  FormContextType,
  getTemplate,
  getUiOptions,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

const ArrayFieldTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: ArrayFieldTemplateProps<T, S, F>,
) => {
  const {
    canAdd,
    disabled,
    fieldPathId,
    uiSchema,
    items,
    onAddClick,
    readonly,
    registry,
    required,
    schema,
    title,
  } = props;
  const uiOptions = getUiOptions<T, S, F>(uiSchema);
  const ArrayFieldDescriptionTemplate = getTemplate<
    "ArrayFieldDescriptionTemplate",
    T,
    S,
    F
  >("ArrayFieldDescriptionTemplate", registry, uiOptions);
  const ArrayFieldTitleTemplate = getTemplate<
    "ArrayFieldTitleTemplate",
    T,
    S,
    F
  >("ArrayFieldTitleTemplate", registry, uiOptions);
  // Button templates are not overridden in the uiSchema
  const {
    ButtonTemplates: { AddButton },
  } = registry.templates;
  return (
    <div>
      <ArrayFieldTitleTemplate
        fieldPathId={fieldPathId}
        title={uiOptions.title || title}
        schema={schema}
        uiSchema={uiSchema}
        required={required}
        registry={registry}
      />
      <ArrayFieldDescriptionTemplate
        fieldPathId={fieldPathId}
        description={uiOptions.description || schema.description}
        schema={schema}
        uiSchema={uiSchema}
        registry={registry}
      />
      <div key={`array-item-list-${fieldPathId.$id}`}>
        {items}
        {canAdd && (
          <AddButton
            onClick={onAddClick}
            disabled={readonly || disabled}
            uiSchema={uiSchema}
            registry={registry}
            className="mx-0 my-2"
          />
        )}
      </div>
    </div>
  );
};

export { ArrayFieldTemplate };
