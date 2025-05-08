import {
  ArrayFieldItemTemplateType,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

const ArrayFieldItemTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: ArrayFieldItemTemplateType<T, S, F>,
) => {
  const {
    children,
    disabled,
    hasToolbar,
    index,
    readonly,
    registry,
    uiSchema,
    buttonsProps,
  } = props;

  const { CopyButton, MoveDownButton, MoveUpButton, RemoveButton } =
    registry.templates.ButtonTemplates;

  const {
    hasCopy,
    hasMoveDown,
    hasMoveUp,
    hasRemove,
    onCopyIndexClick,
    onDropIndexClick,
    onReorderClick,
  } = buttonsProps;

  return (
    <div>
      <div className="mb-2 flex items-center gap-1">
        <div>{children}</div>
        {/* TODO: depending on the type of the children, stylings for icons are broken. */}
        <div className="pt-4">
          {hasToolbar && (
            <div className="flex flex-row gap-1">
              {(hasMoveUp || hasMoveDown) && (
                <div>
                  <MoveUpButton
                    disabled={disabled || readonly || !hasMoveUp}
                    onClick={onReorderClick(index, index - 1)}
                    uiSchema={uiSchema}
                    registry={registry}
                  />
                </div>
              )}
              {(hasMoveUp || hasMoveDown) && (
                <div>
                  <MoveDownButton
                    disabled={disabled || readonly || !hasMoveDown}
                    onClick={onReorderClick(index, index + 1)}
                    uiSchema={uiSchema}
                    registry={registry}
                  />
                </div>
              )}
              {hasCopy && (
                <CopyButton
                  disabled={disabled || readonly}
                  onClick={onCopyIndexClick(index)}
                  uiSchema={uiSchema}
                  registry={registry}
                />
              )}
              {hasRemove && (
                <RemoveButton
                  disabled={disabled || readonly}
                  onClick={onDropIndexClick(index)}
                  uiSchema={uiSchema}
                  registry={registry}
                />
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export { ArrayFieldItemTemplate };
