import {
  canExpand,
  descriptionId,
  FormContextType,
  getTemplate,
  getUiOptions,
  ObjectFieldTemplateProps,
  RJSFSchema,
  StrictRJSFSchema,
  titleId,
} from "@rjsf/utils";

const ObjectFieldTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  description,
  title,
  properties,
  required,
  uiSchema,
  fieldPathId,
  schema,
  formData,
  onAddProperty,
  disabled,
  readonly,
  registry,
}: ObjectFieldTemplateProps<T, S, F>) => {
  const uiOptions = getUiOptions<T, S, F>(uiSchema);
  const TitleFieldTemplate = getTemplate<"TitleFieldTemplate", T, S, F>(
    "TitleFieldTemplate",
    registry,
    uiOptions,
  );
  const DescriptionFieldTemplate = getTemplate<
    "DescriptionFieldTemplate",
    T,
    S,
    F
  >("DescriptionFieldTemplate", registry, uiOptions);
  // Button templates are not overridden in the uiSchema
  const {
    ButtonTemplates: { AddButton },
  } = registry.templates;
  return (
    <>
      {title && (
        <TitleFieldTemplate
          id={titleId(fieldPathId)}
          title={title}
          required={required}
          schema={schema}
          uiSchema={uiSchema}
          registry={registry}
        />
      )}
      {description && (
        <DescriptionFieldTemplate
          id={descriptionId(fieldPathId)}
          description={description}
          schema={schema}
          uiSchema={uiSchema}
          registry={registry}
        />
      )}
      <div>
        {properties.map((element: any, index: number) => (
          <div key={index} className={element.hidden ? "hidden" : undefined}>
            <div> {element.content}</div>
          </div>
        ))}
        {canExpand(schema, uiSchema, formData) ? (
          <div>
            <div className="py-4">
              <AddButton
                onClick={() => onAddProperty()}
                disabled={readonly || disabled}
                uiSchema={uiSchema}
                registry={registry}
              />
            </div>
          </div>
        ) : null}
      </div>
    </>
  );
};

export { ObjectFieldTemplate };
