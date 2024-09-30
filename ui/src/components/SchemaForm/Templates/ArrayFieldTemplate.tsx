import {
  ArrayFieldTemplateItemType,
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
    idSchema,
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
  const ArrayFieldItemTemplate = getTemplate<"ArrayFieldItemTemplate", T, S, F>(
    "ArrayFieldItemTemplate",
    registry,
    uiOptions,
  );
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
      <div>
        <div>
          <ArrayFieldTitleTemplate
            idSchema={idSchema}
            title={uiOptions.title || title}
            schema={schema}
            uiSchema={uiSchema}
            required={required}
            registry={registry}
          />
          <ArrayFieldDescriptionTemplate
            idSchema={idSchema}
            description={uiOptions.description || schema.description}
            schema={schema}
            uiSchema={uiSchema}
            registry={registry}
          />
          <div key={`array-item-list-${idSchema.$id}`}>
            {items &&
              items.map(
                ({
                  key,
                  ...itemProps
                }: ArrayFieldTemplateItemType<T, S, F>) => (
                  <ArrayFieldItemTemplate key={key} {...itemProps} />
                ),
              )}
            {canAdd && (
              <div>
                <div className="mt-2">
                  <div className="py-4">
                    <AddButton
                      onClick={onAddClick}
                      disabled={disabled || readonly}
                      uiSchema={uiSchema}
                      registry={registry}
                    />
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export { ArrayFieldTemplate };
